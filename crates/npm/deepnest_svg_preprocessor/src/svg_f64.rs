//! SVG → absolute outlines, every number kept in `f64`.
//!
//! usvg (and tiny-skia-path below it) store all geometry as `f32`: a 1000 mm box came back
//! 2834.6457 units wide at 72 units/in instead of 2834.645669291339, so a part drawn as large
//! as its sheet no longer fitted (deepnest#137). This reader
//! keeps the file's numbers in `f64` from the attribute to the result and follows usvg's
//! rules for what is drawn: units and `viewBox`/`preserveAspectRatio`, nested transforms,
//! presentation attributes, `style` and `<style>` sheets (simplecss, like usvg),
//! inheritance, `display`/`visibility`, `<switch>` conditions, `<use>` of elements,
//! `<symbol>` and `<svg>`, nested `<svg>`, and the basic shapes. Arcs become cubics
//! (svgtypes/kurbo, in `f64`). Not drawn, like usvg without its text engine: text, images,
//! markers, clip paths and masks (clipping is ignored, the geometry stays whole).

use std::collections::HashMap;
use std::str::FromStr;

use roxmltree::{Document, Node, NodeId};
use svgtypes::{Length, LengthUnit};

const SVG_NS: &str = "http://www.w3.org/2000/svg";
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";
/// Elements drawn, counting every `<use>` copy (usvg's node limit).
const MAX_ELEMENTS: usize = 1_000_000;
/// Element nesting, including `<use>` links; well above real files, low enough for a
/// 1 MB thread stack.
const MAX_DEPTH: u32 = 256;
/// The style properties this reader needs; other CSS declarations are dropped.
const PROPERTIES: [&str; 9] = [
  "transform",
  "transform-origin",
  "fill",
  "stroke",
  "color",
  "display",
  "visibility",
  "stroke-width",
  "font-size",
];

/// What one SVG px (user unit of a root without `viewBox`) and one inch are in the
/// caller's output unit. CSS px at 96 dpi in mm: `Units { px: 25.4 / 96.0, inch: 25.4 }`.
/// Keeping `inch` separate makes absolute sizes exact: `width="1000mm"` is
/// `1000 * (inch / 25.4)`, with no round trip through px.
#[derive(Clone, Copy, Debug)]
pub struct Units {
  pub px: f64,
  pub inch: f64,
}

/// A resolved `fill` or `stroke`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Paint {
  None,
  Color([u8; 3]),
  /// A gradient or pattern.
  Server,
}

/// A path segment in output units, Y down, origin at the canvas' top-left corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Seg {
  Move([f64; 2]),
  Line([f64; 2]),
  Quad([f64; 2], [f64; 2]),
  Cubic([f64; 2], [f64; 2], [f64; 2]),
  Close,
}

#[derive(Clone, Debug)]
pub struct Shape {
  pub segs: Vec<Seg>,
  pub fill: Paint,
  pub stroke: Paint,
  /// False for `visibility="hidden"` and for shapes with neither fill nor stroke.
  pub visible: bool,
  /// Ids of the enclosing `<g>`, `<switch>`, `<a>`, `<svg>` and `<use>` elements,
  /// outermost first (none from inside a `<use>` copy, like usvg).
  pub groups: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Svg {
  /// Canvas size in output units.
  pub width: f64,
  pub height: f64,
  /// The root `width` or `height` is in mm, cm, in, pt or pc.
  pub physical_size: bool,
  pub shapes: Vec<Shape>,
  /// Drawn `<text>` and `<image>` elements, which this reader does not convert.
  pub texts: usize,
  pub images: usize,
}

/// Parse an SVG document.
pub fn parse(text: &str, units: Units) -> Result<Svg, String> {
  let opt = roxmltree::ParsingOptions {
    allow_dtd: true,
    ..Default::default()
  };
  let doc = Document::parse_with_options(text, opt).map_err(|e| e.to_string())?;
  let root = doc.root_element();
  if !is_svg(root) || root.tag_name().name() != "svg" {
    return Err("the root element is not an SVG <svg> element".into());
  }
  let mut r = Reader::new(&doc, units);
  // A root transform that cannot be inverted draws nothing (usvg).
  if let Some(ctx) = r.root(root)? {
    for child in root.children() {
      r.element(child, &ctx, 1)?;
    }
  }
  if r.fit_canvas {
    let (mut w, mut h) = (0.0f64, 0.0f64);
    for seg in r.out.shapes.iter().flat_map(|s| &s.segs) {
      for p in seg_points(seg) {
        w = w.max(p[0]);
        h = h.max(p[1]);
      }
    }
    if w > 0.0 && h > 0.0 {
      r.out.width = w;
      r.out.height = h;
    }
  }
  Ok(r.out)
}

fn seg_points(seg: &Seg) -> Vec<[f64; 2]> {
  match *seg {
    Seg::Move(a) | Seg::Line(a) => vec![a],
    Seg::Quad(a, b) => vec![a, b],
    Seg::Cubic(a, b, c) => vec![a, b, c],
    Seg::Close => vec![],
  }
}

type Ts = [f64; 6];
const IDENTITY: Ts = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// `m` after `n`: apply `n` first.
fn mul(m: Ts, n: Ts) -> Ts {
  [
    m[0] * n[0] + m[2] * n[1],
    m[1] * n[0] + m[3] * n[1],
    m[0] * n[2] + m[2] * n[3],
    m[1] * n[2] + m[3] * n[3],
    m[0] * n[4] + m[2] * n[5] + m[4],
    m[1] * n[4] + m[3] * n[5] + m[5],
  ]
}

fn apply(m: &Ts, x: f64, y: f64) -> [f64; 2] {
  [m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]]
}

fn is_svg(n: Node) -> bool {
  n.is_element() && n.tag_name().namespace() == Some(SVG_NS)
}

#[derive(Clone, Copy)]
enum Axis {
  X,
  Y,
  Diagonal,
}

#[derive(Clone, Copy, PartialEq)]
enum Fill {
  None,
  Color([u8; 3]),
  CurrentColor,
  Server,
}

/// Computed (inherited) style.
#[derive(Clone, Copy)]
struct Style {
  fill: Fill,
  stroke: Fill,
  color: [u8; 3],
  visible: bool,
  stroke_width: f64,
  font_size: f64,
}

#[derive(Clone, Copy)]
struct Ctx {
  ts: Ts,
  style: Style,
  /// Viewport size in user units, for percentages.
  vp: (f64, f64),
  /// Inside a `<use>` copy: group ids are not reported.
  in_use: bool,
  /// Set on the element a `<use>` links to: the `<use>`'s `width`/`height`, if given.
  link: Option<(Option<f64>, Option<f64>)>,
  /// `context-fill`/`context-stroke`: the fill and stroke of the nearest `<use>`.
  context: (Fill, Fill),
}

struct Reader<'a, 'i> {
  ids: HashMap<&'a str, Node<'a, 'i>>,
  /// Stylesheet declarations, then `style` attribute declarations, per element.
  css: HashMap<NodeId, Vec<simplecss::Declaration<'a>>>,
  units: Units,
  /// px per inch, for absolute lengths inside the drawing.
  dpi: f64,
  out: Svg,
  groups: Vec<String>,
  links: Vec<NodeId>,
  elements: usize,
  /// Root size in `%` without a `viewBox`: the canvas becomes the content's extent.
  fit_canvas: bool,
}

impl<'a, 'i: 'a> Reader<'a, 'i> {
  fn new(doc: &'a Document<'i>, units: Units) -> Self {
    let mut sheet = simplecss::StyleSheet::new();
    for node in doc.descendants().filter(|n| n.has_tag_name("style")) {
      if node.attribute("type").is_some_and(|t| t != "text/css") {
        continue;
      }
      if let Some(text) = node.text() {
        sheet.parse_more(text);
      }
    }
    let mut ids = HashMap::new();
    let mut css: HashMap<NodeId, Vec<simplecss::Declaration<'a>>> = HashMap::new();
    for node in doc.descendants().filter(|n| n.is_element()) {
      if let Some(id) = node.attribute("id") {
        ids.entry(id).or_insert(node);
      }
      let mut decls = Vec::new();
      for rule in &sheet.rules {
        if rule.selector.matches(&El(node)) {
          decls.extend(rule.declarations.iter().copied());
        }
      }
      if let Some(style) = node.attribute("style") {
        decls.extend(simplecss::DeclarationTokenizer::from(style));
      }
      decls.retain(|d| PROPERTIES.contains(&d.name));
      if !decls.is_empty() {
        css.insert(node.id(), decls);
      }
    }
    Reader {
      ids,
      css,
      units,
      dpi: units.inch / units.px,
      out: Svg::default(),
      groups: Vec::new(),
      links: Vec::new(),
      elements: 0,
      fit_canvas: false,
    }
  }

  /// Canvas size and the root transform (user units → output units).
  fn root(&mut self, root: Node<'a, 'i>) -> Result<Option<Ctx>, String> {
    let u = self.units;
    let vb = view_box(root);
    let mut fallback = false;
    let mut size = |name: &str, vb_len: Option<f64>| -> Option<f64> {
      let l = root
        .attribute(name)
        .and_then(|v| Length::from_str(v).ok())
        .unwrap_or(Length::new(100.0, LengthUnit::Percent));
      let n = l.number;
      let v = match l.unit {
        LengthUnit::Percent => match vb_len {
          Some(v) => v * (n / 100.0) * u.px,
          // usvg: 100 px, then the content's bounding box.
          None => {
            fallback = true;
            n * u.px
          }
        },
        LengthUnit::None | LengthUnit::Px => n * u.px,
        LengthUnit::In => n * u.inch,
        LengthUnit::Cm => n * (u.inch / 2.54),
        LengthUnit::Mm => n * (u.inch / 25.4),
        LengthUnit::Pt => n * (u.inch / 72.0),
        LengthUnit::Pc => n * (u.inch / 6.0),
        LengthUnit::Em => n * 12.0 * u.px,
        LengthUnit::Ex => n * 6.0 * u.px,
      };
      (v.is_finite() && v > 0.0).then_some(v)
    };
    let w = size("width", vb.map(|v| v.w));
    let h = size("height", vb.map(|v| v.h));
    let (Some(w), Some(h)) = (w, h) else {
      return Err("SVG has an invalid size".into());
    };
    self.out.width = w;
    self.out.height = h;
    self.out.physical_size = ["width", "height"].iter().any(|a| {
      root
        .attribute(*a)
        .and_then(|v| Length::from_str(v).ok())
        .is_some_and(|l| {
          matches!(
            l.unit,
            LengthUnit::Mm | LengthUnit::Cm | LengthUnit::In | LengthUnit::Pt | LengthUnit::Pc
          )
        })
    });
    let (ts, vp) = match vb {
      Some(vb) => (view_box_ts(&vb, aspect(root), w, h), (vb.w, vb.h)),
      None => ([u.px, 0.0, 0.0, u.px, 0.0, 0.0], (w / u.px, h / u.px)),
    };
    let style = Style {
      fill: Fill::Color([0, 0, 0]),
      stroke: Fill::None,
      color: [0, 0, 0],
      visible: true,
      stroke_width: 1.0,
      font_size: 12.0,
    };
    let mut ctx = Ctx {
      ts,
      style,
      vp,
      in_use: false,
      link: None,
      context: (Fill::None, Fill::None),
    };
    ctx.style = self.style(root, &ctx);
    self.fit_canvas = fallback;
    if self.decl(root, "display") == Some("none") || !conditions_pass(root) {
      return Ok(None);
    }
    // The root's transform turns around the canvas centre and applies after the
    // `viewBox` (usvg, CSS initial `transform-origin`).
    let canvas = Ctx {
      ts: IDENTITY,
      vp: (w / u.px, h / u.px),
      ..ctx
    };
    Ok(self.transform(root, &canvas, (50.0, 50.0)).map(|t| {
      let t = [t[0], t[1], t[2], t[3], t[4] * u.px, t[5] * u.px];
      ctx.ts = mul(t, ctx.ts);
      ctx
    }))
  }

  fn element(&mut self, node: Node<'a, 'i>, ctx: &Ctx, depth: u32) -> Result<(), String> {
    if !is_svg(node) {
      return Ok(());
    }
    self.elements += 1;
    if self.elements > MAX_ELEMENTS {
      return Err(format!(
        "SVG has more than {MAX_ELEMENTS} elements (counting <use> copies)"
      ));
    }
    if depth > MAX_DEPTH {
      return Err(format!("SVG elements are nested deeper than {MAX_DEPTH}"));
    }
    if self.decl(node, "display") == Some("none") || !conditions_pass(node) {
      return Ok(());
    }
    // A transform that cannot be inverted draws nothing (usvg).
    let Some(own) = self.transform(node, ctx, (0.0, 0.0)) else {
      return Ok(());
    };
    let mut c = Ctx {
      ts: mul(ctx.ts, own),
      link: None,
      ..*ctx
    };
    c.style = self.style(node, &c);
    let name = node.tag_name().name();
    match name {
      "g" | "a" | "switch" | "svg" | "use" | "symbol" => {
        let pushed = !c.in_use && self.push_group(node);
        match name {
          "switch" => {
            let first = node.children().find(|n| {
              is_svg(*n) && DRAWABLE.contains(&n.tag_name().name()) && conditions_pass(*n)
            });
            if let Some(child) = first {
              self.element(child, &c, depth + 1)?;
            }
          }
          "use" => self.use_element(node, c, depth)?,
          "svg" => {
            let link = ctx.link.unwrap_or_default();
            let x = self.len(node, "x", Axis::X, &c, 0.0);
            let y = self.len(node, "y", Axis::Y, &c, 0.0);
            let w = link
              .0
              .unwrap_or_else(|| self.len(node, "width", Axis::X, &c, -1.0));
            let h = link
              .1
              .unwrap_or_else(|| self.len(node, "height", Axis::Y, &c, -1.0));
            // Width and height default to 100 %.
            let w = if w < 0.0 { c.vp.0 } else { w };
            let h = if h < 0.0 { c.vp.1 } else { h };
            c.ts = mul(c.ts, [1.0, 0.0, 0.0, 1.0, x, y]);
            self.viewport(node, w, h, c, depth)?;
          }
          // Drawn only through a `<use>`, which already moved it to its x/y.
          "symbol" => {
            if let Some(link) = ctx.link {
              let (w, h) = (link.0.unwrap_or(c.vp.0), link.1.unwrap_or(c.vp.1));
              self.viewport(node, w, h, c, depth)?;
            }
          }
          _ => {
            for child in node.children() {
              self.element(child, &c, depth + 1)?;
            }
          }
        }
        if pushed {
          self.groups.pop();
        }
      }
      "rect" | "circle" | "ellipse" | "line" | "polyline" | "polygon" | "path" => {
        self.shape(node, name, &c);
      }
      "text" => self.out.texts += 1,
      "image" => self.out.images += 1,
      _ => {}
    }
    Ok(())
  }

  fn push_group(&mut self, node: Node) -> bool {
    match node.attribute("id") {
      Some(id) if !id.is_empty() => {
        self.groups.push(id.to_string());
        true
      }
      _ => false,
    }
  }

  /// The children of a nested `<svg>` or a linked `<symbol>`, `w` × `h` user units big.
  fn viewport(
    &mut self,
    node: Node<'a, 'i>,
    w: f64,
    h: f64,
    mut c: Ctx,
    depth: u32,
  ) -> Result<(), String> {
    c.vp = (w, h);
    if let Some(vb) = view_box(node) {
      if w > 0.0 && h > 0.0 {
        c.ts = mul(c.ts, view_box_ts(&vb, aspect(node), w, h));
      }
      if node.tag_name().name() == "svg" {
        c.vp = (vb.w, vb.h);
      }
    }
    for child in node.children() {
      self.element(child, &c, depth + 1)?;
    }
    Ok(())
  }

  fn use_element(&mut self, node: Node<'a, 'i>, mut c: Ctx, depth: u32) -> Result<(), String> {
    let href = node
      .attribute((XLINK_NS, "href"))
      .or_else(|| node.attribute("href"));
    // Only links inside the document; external files are never loaded.
    let Some(target) = href
      .and_then(|h| h.trim().strip_prefix('#'))
      .and_then(|id| self.ids.get(id).copied())
    else {
      return Ok(());
    };
    // A link to itself, to an ancestor, or around a loop of `<use>` draws nothing.
    if node.ancestors().any(|a| a == target) || self.links.contains(&target.id()) {
      return Ok(());
    }
    let x = self.len(node, "x", Axis::X, &c, 0.0);
    let y = self.len(node, "y", Axis::Y, &c, 0.0);
    let size = |name, axis| {
      node
        .has_attribute(name)
        .then(|| self.len(node, name, axis, &c, 0.0))
    };
    let link = (size("width", Axis::X), size("height", Axis::Y));
    c.ts = mul(c.ts, [1.0, 0.0, 0.0, 1.0, x, y]);
    c.in_use = true;
    c.link = Some(link);
    c.context = (c.style.fill, c.style.stroke);
    self.links.push(target.id());
    let r = self.element(target, &c, depth + 1);
    self.links.pop();
    r
  }

  fn shape(&mut self, node: Node<'a, 'i>, name: &str, c: &Ctx) {
    let Some(segs) = self.outline(node, name, c) else {
      return;
    };
    let mut out = Vec::with_capacity(segs.len());
    let mut drawn = false;
    for seg in segs {
      let t = |x, y| apply(&c.ts, x, y);
      let s = match seg {
        svgtypes::SimplePathSegment::MoveTo { x, y } => Seg::Move(t(x, y)),
        svgtypes::SimplePathSegment::LineTo { x, y } => Seg::Line(t(x, y)),
        svgtypes::SimplePathSegment::Quadratic { x1, y1, x, y } => Seg::Quad(t(x1, y1), t(x, y)),
        svgtypes::SimplePathSegment::CurveTo {
          x1,
          y1,
          x2,
          y2,
          x,
          y,
        } => Seg::Cubic(t(x1, y1), t(x2, y2), t(x, y)),
        svgtypes::SimplePathSegment::ClosePath => Seg::Close,
      };
      if seg_points(&s)
        .iter()
        .any(|p| !p[0].is_finite() || !p[1].is_finite())
      {
        return;
      }
      drawn |= !matches!(s, Seg::Move(_) | Seg::Close);
      out.push(s);
    }
    if !drawn {
      return;
    }
    let st = &c.style;
    let fill = paint(st.fill, st.color);
    let stroke = if st.stroke_width > 0.0 {
      paint(st.stroke, st.color)
    } else {
      Paint::None
    };
    self.out.shapes.push(Shape {
      segs: out,
      fill,
      stroke,
      visible: st.visible && (fill != Paint::None || stroke != Paint::None),
      groups: self.groups.clone(),
    });
  }

  /// A shape's outline in its own user units (usvg's rules for invalid sizes).
  fn outline(
    &self,
    node: Node<'a, 'i>,
    name: &str,
    c: &Ctx,
  ) -> Option<Vec<svgtypes::SimplePathSegment>> {
    use svgtypes::SimplePathSegment as S;
    let len = |attr, axis| self.len(node, attr, axis, c, 0.0);
    let d = match name {
      "path" => return Some(path_data(node.attribute("d")?)),
      "rect" => {
        let (w, h) = (len("width", Axis::X), len("height", Axis::Y));
        if !(w > 0.0 && h > 0.0) {
          return None;
        }
        let (x, y) = (len("x", Axis::X), len("y", Axis::Y));
        let (rx, ry) = self.radii(node, c);
        let (rx, ry) = (rx.min(w / 2.0), ry.min(h / 2.0));
        if rx == 0.0 {
          return Some(vec![
            S::MoveTo { x, y },
            S::LineTo { x: x + w, y },
            S::LineTo { x: x + w, y: y + h },
            S::LineTo { x, y: y + h },
            S::ClosePath,
          ]);
        }
        // f64 `Display` round-trips, so the path text keeps every bit.
        format!(
          "M {} {y} L {} {y} A {rx} {ry} 0 0 1 {} {} L {} {} A {rx} {ry} 0 0 1 {} {} \
                     L {} {} A {rx} {ry} 0 0 1 {x} {} L {x} {} A {rx} {ry} 0 0 1 {} {y} Z",
          x + rx,
          x + w - rx,
          x + w,
          y + ry,
          x + w,
          y + h - ry,
          x + w - rx,
          y + h,
          x + rx,
          y + h,
          y + h - ry,
          y + ry,
          x + rx,
        )
      }
      "circle" | "ellipse" => {
        let (cx, cy) = (len("cx", Axis::X), len("cy", Axis::Y));
        let (rx, ry) = if name == "circle" {
          let r = len("r", Axis::Diagonal);
          (r, r)
        } else {
          self.radii(node, c)
        };
        if !(rx > 0.0 && ry > 0.0) {
          return None;
        }
        format!(
          "M {} {cy} A {rx} {ry} 0 0 1 {cx} {} A {rx} {ry} 0 0 1 {} {cy} \
                     A {rx} {ry} 0 0 1 {cx} {} A {rx} {ry} 0 0 1 {} {cy} Z",
          cx + rx,
          cy + ry,
          cx - rx,
          cy - ry,
          cx + rx,
        )
      }
      "line" => {
        return Some(vec![
          S::MoveTo {
            x: len("x1", Axis::X),
            y: len("y1", Axis::Y),
          },
          S::LineTo {
            x: len("x2", Axis::X),
            y: len("y2", Axis::Y),
          },
        ]);
      }
      _ => {
        let mut segs: Vec<S> = svgtypes::PointsParser::from(node.attribute("points")?)
          .enumerate()
          .map(|(i, (x, y))| {
            if i == 0 {
              S::MoveTo { x, y }
            } else {
              S::LineTo { x, y }
            }
          })
          .collect();
        if segs.len() < 2 {
          return None;
        }
        if name == "polygon" {
          segs.push(S::ClosePath);
        }
        return Some(segs);
      }
    };
    Some(path_data(&d))
  }

  /// `rx`/`ry` of a rect or ellipse: a missing or negative one takes the other's value.
  fn radii(&self, node: Node<'a, 'i>, c: &Ctx) -> (f64, f64) {
    let get = |name, axis| {
      node
        .attribute(name)
        .and_then(|v| Length::from_str(v).ok())
        .filter(|l| !l.number.is_sign_negative())
        .map(|l| self.length(l, axis, c))
    };
    match (get("rx", Axis::X), get("ry", Axis::Y)) {
      (None, None) => (0.0, 0.0),
      (Some(rx), None) => (rx, rx),
      (None, Some(ry)) => (ry, ry),
      (Some(rx), Some(ry)) => (rx, ry),
    }
  }

  /// The cascaded value of a style property on `node`: presentation attribute < `<style>`
  /// rules in order < `style` attribute, and the first `!important` one wins (usvg).
  fn decl(&self, node: Node<'a, 'i>, name: &str) -> Option<&'a str> {
    let mut best = node.attribute(name).map(|v| (v, false));
    for d in self.css.get(&node.id()).into_iter().flatten() {
      if d.name == name && !best.is_some_and(|b| b.1) {
        best = Some((d.value, d.important));
      }
    }
    best.map(|b| b.0.trim())
  }

  fn style(&self, node: Node<'a, 'i>, c: &Ctx) -> Style {
    let parent = c.style;
    let mut st = parent;
    if let Some(v) = self.decl(node, "color") {
      if let Ok(col) = svgtypes::Color::from_str(v) {
        st.color = [col.red, col.green, col.blue];
      }
    }
    if let Some(f) = self.decl(node, "fill").and_then(|v| self.fill(v, c)) {
      st.fill = f;
    }
    if let Some(f) = self.decl(node, "stroke").and_then(|v| self.fill(v, c)) {
      st.stroke = f;
    }
    match self.decl(node, "visibility") {
      Some("visible") => st.visible = true,
      Some("hidden" | "collapse") => st.visible = false,
      _ => {}
    }
    if let Some(l) = self
      .decl(node, "font-size")
      .and_then(|v| Length::from_str(v).ok())
    {
      st.font_size = match l.unit {
        LengthUnit::Em => l.number * parent.font_size,
        LengthUnit::Ex => l.number * parent.font_size / 2.0,
        LengthUnit::Percent => l.number / 100.0 * parent.font_size,
        _ => self.length(l, Axis::Diagonal, c),
      };
    }
    if let Some(l) = self
      .decl(node, "stroke-width")
      .and_then(|v| Length::from_str(v).ok())
    {
      let c = Ctx { style: st, ..*c };
      st.stroke_width = self.length(l, Axis::Diagonal, &c);
    }
    st
  }

  /// A `fill`/`stroke` value; `None` keeps the inherited one.
  fn fill(&self, value: &str, c: &Ctx) -> Option<Fill> {
    use svgtypes::{Paint as P, PaintFallback as F};
    let color = |c: svgtypes::Color| Fill::Color([c.red, c.green, c.blue]);
    match P::from_str(value).ok()? {
      P::None => Some(Fill::None),
      P::Inherit => None,
      P::CurrentColor => Some(Fill::CurrentColor),
      P::Color(c) => Some(color(c)),
      P::FuncIRI(id, fallback) => {
        let server = self
          .ids
          .get(id)
          .is_some_and(|n| paint_server(*n, &self.ids));
        Some(match (server, fallback) {
          (true, _) => Fill::Server,
          (false, Some(F::Color(c))) => color(c),
          (false, Some(F::CurrentColor)) => Fill::CurrentColor,
          (false, _) => Fill::None,
        })
      }
      P::ContextFill => Some(c.context.0),
      P::ContextStroke => Some(c.context.1),
    }
  }

  /// `transform` around `transform-origin` (default `origin`, in % of the viewport);
  /// `None` if it cannot be inverted.
  fn transform(&self, node: Node<'a, 'i>, c: &Ctx, origin: (f64, f64)) -> Option<Ts> {
    let Some(t) = self
      .decl(node, "transform")
      .and_then(|v| svgtypes::Transform::from_str(v).ok())
    else {
      return Some(IDENTITY);
    };
    let det = t.a * t.d - t.b * t.c;
    if ![t.a, t.b, t.c, t.d, t.e, t.f, det]
      .iter()
      .all(|v| v.is_finite())
      || det == 0.0
    {
      return None;
    }
    let ts = [t.a, t.b, t.c, t.d, t.e, t.f];
    let (ox, oy) = match self
      .decl(node, "transform-origin")
      .and_then(|v| svgtypes::TransformOrigin::from_str(v).ok())
    {
      Some(o) => (
        self.length(o.x_offset, Axis::X, c),
        self.length(o.y_offset, Axis::Y, c),
      ),
      None => (origin.0 / 100.0 * c.vp.0, origin.1 / 100.0 * c.vp.1),
    };
    if ox == 0.0 && oy == 0.0 {
      return Some(ts);
    }
    let to = |x, y| [1.0, 0.0, 0.0, 1.0, x, y];
    Some(mul(mul(to(ox, oy), ts), to(-ox, -oy)))
  }

  fn len(&self, node: Node<'a, 'i>, attr: &str, axis: Axis, c: &Ctx, default: f64) -> f64 {
    node
      .attribute(attr)
      .and_then(|v| Length::from_str(v).ok())
      .map_or(default, |l| self.length(l, axis, c))
  }

  /// A length in user units.
  fn length(&self, l: Length, axis: Axis, c: &Ctx) -> f64 {
    let n = l.number;
    match l.unit {
      LengthUnit::None | LengthUnit::Px => n,
      LengthUnit::Em => n * c.style.font_size,
      LengthUnit::Ex => n * c.style.font_size / 2.0,
      LengthUnit::In => n * self.dpi,
      LengthUnit::Cm => n * (self.dpi / 2.54),
      LengthUnit::Mm => n * (self.dpi / 25.4),
      LengthUnit::Pt => n * (self.dpi / 72.0),
      LengthUnit::Pc => n * (self.dpi / 6.0),
      LengthUnit::Percent => {
        let (w, h) = c.vp;
        n / 100.0
          * match axis {
            Axis::X => w,
            Axis::Y => h,
            Axis::Diagonal => ((w * w + h * h) / 2.0).sqrt(),
          }
      }
    }
  }
}

/// A pattern, or a gradient with stops (its own or through `href`). Anything else gets
/// the fallback paint, or none (usvg).
fn paint_server(n: Node, ids: &HashMap<&str, Node>) -> bool {
  let mut n = n;
  for _ in 0..16 {
    let name = n.tag_name().name();
    if !is_svg(n) || !matches!(name, "pattern" | "linearGradient" | "radialGradient") {
      return false;
    }
    if name == "pattern"
      || n
        .children()
        .any(|c| is_svg(c) && c.tag_name().name() == "stop")
    {
      return true;
    }
    let href = n
      .attribute((XLINK_NS, "href"))
      .or_else(|| n.attribute("href"));
    match href
      .and_then(|h| h.trim().strip_prefix('#'))
      .and_then(|id| ids.get(id))
    {
      Some(next) => n = *next,
      None => return false,
    }
  }
  false
}

fn paint(f: Fill, color: [u8; 3]) -> Paint {
  match f {
    Fill::None => Paint::None,
    Fill::Color(c) => Paint::Color(c),
    Fill::CurrentColor => Paint::Color(color),
    Fill::Server => Paint::Server,
  }
}

/// Path data up to the first error, like usvg: absolute, `H`/`V`/`S`/`T` spelled out,
/// arcs as cubics (svgtypes/kurbo) that end exactly on the arc's end point.
fn path_data(d: &str) -> Vec<svgtypes::SimplePathSegment> {
  use svgtypes::{PathSegment as P, SimplePathSegment as S};
  let mut out = Vec::new();
  let (mut cur, mut start) = ((0.0, 0.0), (0.0, 0.0));
  // The last cubic's second / quadratic's control point, for `S` / `T`.
  let (mut cubic_ctrl, mut quad_ctrl) = (None, None);
  let mut after_close = false;
  for seg in svgtypes::PathParser::from(d) {
    let Ok(seg) = seg else { break };
    let (c, q) = (cubic_ctrl.take(), quad_ctrl.take());
    let at = |abs: bool, x: f64, y: f64| if abs { (x, y) } else { (cur.0 + x, cur.1 + y) };
    let reflect = |p: Option<(f64, f64)>| p.map_or(cur, |p| (2.0 * cur.0 - p.0, 2.0 * cur.1 - p.1));
    // A command after `Z` other than `M` starts a new subpath at the old start.
    if after_close && !matches!(seg, P::MoveTo { .. } | P::ClosePath { .. }) {
      out.push(S::MoveTo {
        x: start.0,
        y: start.1,
      });
    }
    after_close = false;
    let end = match seg {
      P::MoveTo { abs, x, y } => {
        let p = at(abs, x, y);
        start = p;
        out.push(S::MoveTo { x: p.0, y: p.1 });
        p
      }
      P::LineTo { abs, x, y } => at(abs, x, y),
      P::HorizontalLineTo { abs, x } => (if abs { x } else { cur.0 + x }, cur.1),
      P::VerticalLineTo { abs, y } => (cur.0, if abs { y } else { cur.1 + y }),
      P::CurveTo {
        abs,
        x1,
        y1,
        x2,
        y2,
        x,
        y,
      } => {
        let (p1, p2, p) = (at(abs, x1, y1), at(abs, x2, y2), at(abs, x, y));
        out.push(S::CurveTo {
          x1: p1.0,
          y1: p1.1,
          x2: p2.0,
          y2: p2.1,
          x: p.0,
          y: p.1,
        });
        cubic_ctrl = Some(p2);
        p
      }
      P::SmoothCurveTo { abs, x2, y2, x, y } => {
        let (p1, p2, p) = (reflect(c), at(abs, x2, y2), at(abs, x, y));
        out.push(S::CurveTo {
          x1: p1.0,
          y1: p1.1,
          x2: p2.0,
          y2: p2.1,
          x: p.0,
          y: p.1,
        });
        cubic_ctrl = Some(p2);
        p
      }
      P::Quadratic { abs, x1, y1, x, y } => {
        let (p1, p) = (at(abs, x1, y1), at(abs, x, y));
        out.push(S::Quadratic {
          x1: p1.0,
          y1: p1.1,
          x: p.0,
          y: p.1,
        });
        quad_ctrl = Some(p1);
        p
      }
      P::SmoothQuadratic { abs, x, y } => {
        let (p1, p) = (reflect(q), at(abs, x, y));
        out.push(S::Quadratic {
          x1: p1.0,
          y1: p1.1,
          x: p.0,
          y: p.1,
        });
        quad_ctrl = Some(p1);
        p
      }
      P::EllipticalArc {
        abs,
        rx,
        ry,
        x_axis_rotation,
        large_arc,
        sweep,
        x,
        y,
      } => {
        let p = at(abs, x, y);
        // f64 `Display` round-trips, so the arc text keeps every bit.
        let arc = format!(
          "M {} {} A {rx} {ry} {x_axis_rotation} {} {} {} {}",
          cur.0, cur.1, large_arc as u8, sweep as u8, p.0, p.1
        );
        let n = out.len();
        out.extend(
          svgtypes::SimplifyingPathParser::from(arc.as_str())
            .skip(1)
            .map_while(Result::ok),
        );
        if let Some(S::CurveTo { x, y, .. } | S::LineTo { x, y }) =
          out.get_mut(n..).and_then(|s| s.last_mut())
        {
          (*x, *y) = p;
        }
        cur = p;
        continue;
      }
      P::ClosePath { .. } => {
        out.push(S::ClosePath);
        after_close = true;
        start
      }
    };
    if matches!(
      seg,
      P::LineTo { .. } | P::HorizontalLineTo { .. } | P::VerticalLineTo { .. }
    ) {
      out.push(S::LineTo { x: end.0, y: end.1 });
    }
    cur = end;
  }
  out
}

/// Elements a `<switch>` can pick (unknown elements are skipped, like usvg).
const DRAWABLE: [&str; 15] = [
  "a",
  "circle",
  "ellipse",
  "foreignObject",
  "g",
  "image",
  "line",
  "path",
  "polygon",
  "polyline",
  "rect",
  "svg",
  "switch",
  "text",
  "use",
];

fn view_box(node: Node) -> Option<svgtypes::ViewBox> {
  node
    .attribute("viewBox")
    .and_then(|v| svgtypes::ViewBox::from_str(v).ok())
    .filter(|vb| vb.w > 0.0 && vb.h > 0.0)
}

fn aspect(node: Node) -> svgtypes::AspectRatio {
  node
    .attribute("preserveAspectRatio")
    .and_then(|v| svgtypes::AspectRatio::from_str(v).ok())
    .unwrap_or_default()
}

/// Maps `vb` onto a `w` × `h` viewport.
fn view_box_ts(vb: &svgtypes::ViewBox, ar: svgtypes::AspectRatio, w: f64, h: f64) -> Ts {
  use svgtypes::Align as A;
  let (sx, sy) = (w / vb.w, h / vb.h);
  if ar.align == A::None {
    return [sx, 0.0, 0.0, sy, -vb.x * sx, -vb.y * sy];
  }
  let s = if ar.slice { sx.max(sy) } else { sx.min(sy) };
  let (fx, fy) = match ar.align {
    A::XMinYMin => (0.0, 0.0),
    A::XMidYMin => (0.5, 0.0),
    A::XMaxYMin => (1.0, 0.0),
    A::XMinYMid => (0.0, 0.5),
    A::XMaxYMid => (1.0, 0.5),
    A::XMinYMax => (0.0, 1.0),
    A::XMidYMax => (0.5, 1.0),
    A::XMaxYMax => (1.0, 1.0),
    _ => (0.5, 0.5),
  };
  [
    s,
    0.0,
    0.0,
    s,
    -vb.x * s + (w - vb.w * s) * fx,
    -vb.y * s + (h - vb.h * s) * fy,
  ]
}

/// `requiredExtensions`, `requiredFeatures` and `systemLanguage` (English), like usvg.
fn conditions_pass(node: Node) -> bool {
  const FEATURES: [&str; 26] = [
    "SVGDOM-static",
    "SVG-static",
    "CoreAttribute",
    "Structure",
    "BasicStructure",
    "ContainerAttribute",
    "ConditionalProcessing",
    "Image",
    "Style",
    "Shape",
    "Text",
    "BasicText",
    "PaintAttribute",
    "BasicPaintAttribute",
    "OpacityAttribute",
    "GraphicsAttribute",
    "BasicGraphicsAttribute",
    "Marker",
    "Gradient",
    "Pattern",
    "Clip",
    "BasicClip",
    "Mask",
    "Filter",
    "BasicFilter",
    "XlinkAttribute",
  ];
  if node.has_attribute("requiredExtensions") {
    return false;
  }
  if let Some(features) = node.attribute("requiredFeatures") {
    let known = |f: &str| {
      f.strip_prefix("http://www.w3.org/TR/SVG11/feature#")
        .is_some_and(|f| FEATURES.contains(&f))
    };
    if !features.split(' ').all(known) {
      return false;
    }
  }
  if let Some(langs) = node.attribute("systemLanguage") {
    let en = |l: &str| {
      let l = l.trim();
      l == "en" || l.starts_with("en-")
    };
    if !langs.split(',').any(en) {
      return false;
    }
  }
  true
}

/// A roxmltree element for simplecss selectors.
struct El<'a, 'i>(Node<'a, 'i>);

impl<'a, 'i> simplecss::Element for El<'a, 'i> {
  fn parent_element(&self) -> Option<Self> {
    self.0.parent_element().map(El)
  }

  fn prev_sibling_element(&self) -> Option<Self> {
    self.0.prev_sibling_element().map(El)
  }

  fn has_local_name(&self, name: &str) -> bool {
    self.0.tag_name().name() == name
  }

  fn attribute_matches(&self, name: &str, op: simplecss::AttributeOperator) -> bool {
    self.0.attribute(name).is_some_and(|v| op.matches(v))
  }

  fn pseudo_class_matches(&self, class: simplecss::PseudoClass) -> bool {
    class == simplecss::PseudoClass::FirstChild && self.0.prev_sibling_element().is_none()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const MM: Units = Units {
    px: 25.4 / 96.0,
    inch: 25.4,
  };
  /// deepnest v1: 72 units per inch, px = unit.
  const PT: Units = Units {
    px: 1.0,
    inch: 72.0,
  };

  fn svg(attrs: &str, body: &str) -> String {
    format!(
      r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" {attrs}>{body}</svg>"#
    )
  }

  fn points(s: &Shape) -> Vec<[f64; 2]> {
    s.segs.iter().flat_map(seg_points).collect()
  }

  fn bbox(s: &Shape) -> [f64; 4] {
    points(s)
      .iter()
      .fold([f64::MAX, f64::MAX, f64::MIN, f64::MIN], |b, p| {
        [
          b[0].min(p[0]),
          b[1].min(p[1]),
          b[2].max(p[0]),
          b[3].max(p[1]),
        ]
      })
  }

  #[test]
  fn mm_sizes_are_exact() {
    // usvg returned 1000.0000076 and 3000.0000875 x 1500.0000437 mm here (f32).
    for (w, h) in [(1000.0, 1000.0), (3000.0, 1500.0), (123.456, 78.9)] {
      let text = svg(
        &format!(r#"width="{w}mm" height="{h}mm" viewBox="0 0 {w} {h}""#),
        &format!(r#"<rect width="{w}" height="{h}"/>"#),
      );
      let mm = parse(&text, MM).unwrap();
      assert_eq!((mm.width, mm.height), (w, h));
      assert_eq!(bbox(&mm.shapes[0]), [0.0, 0.0, w, h]);
      // deepnest v1 types a sheet in mm as `mm * (72 / 25.4)`.
      let pt = parse(&text, PT).unwrap();
      let k = 72.0 / 25.4;
      assert_eq!(bbox(&pt.shapes[0]), [0.0, 0.0, w * k, h * k]);
    }
    let pt = parse(
      &svg(
        r#"width="1000mm" height="1000mm" viewBox="0 0 1000 1000""#,
        r#"<rect width="1000" height="1000"/>"#,
      ),
      PT,
    )
    .unwrap();
    assert_eq!(bbox(&pt.shapes[0])[2], 2834.645669291339);
  }

  #[test]
  fn non_integer_coordinates_and_nested_transforms_stay_f64() {
    let text = svg(
      r#"width="200mm" height="100mm" viewBox="0 0 200 100""#,
      r#"<g transform="translate(10.123456789 20.987654321)"><g transform="matrix(2 0 0 3 0.5 0.25)">
               <path d="M 0.1 0.2 L 33.333333333 44.444444444 l 0.000001 0"/></g></g>"#,
    );
    let s = parse(&text, MM).unwrap();
    let p = points(&s.shapes[0]);
    let x = |v: f64| 10.123456789 + (2.0 * v + 0.5);
    let y = |v: f64| 20.987654321 + (3.0 * v + 0.25);
    assert_eq!(p[0], [x(0.1), y(0.2)]);
    assert_eq!(p[1], [x(33.333333333), y(44.444444444)]);
    assert_eq!(p[2], [x(33.333333333 + 0.000001), y(44.444444444)]);
    // The same in f32 is visibly off: this is what the test guards.
    assert_ne!(p[1][0] as f32 as f64, p[1][0]);
  }

  #[test]
  fn units_viewbox_and_aspect_ratio() {
    // 2 in wide, viewBox 0 0 100 50 → 72 units per 50 user units; xMidYMid meet centres.
    let s = parse(
            &svg(
                r#"width="2in" height="2in" viewBox="0 0 100 50""#,
                r#"<rect x="0" y="0" width="100" height="50"/><rect x="10" y="0" width="1cm" height="10pt"/>"#,
            ),
            PT,
        )
        .unwrap();
    assert_eq!((s.width, s.height), (144.0, 144.0));
    assert!(s.physical_size);
    assert_eq!(bbox(&s.shapes[0]), [0.0, 36.0, 144.0, 108.0]);
    // 1cm and 10pt inside the drawing are user px at 72 px/in, then scaled by 1.44.
    let b = bbox(&s.shapes[1]);
    assert!((b[2] - b[0] - 72.0 / 2.54 * 1.44).abs() < 1e-12);
    assert!((b[3] - b[1] - 10.0 * 1.44).abs() < 1e-12);
    let unitless = parse(&svg(r#"width="96" height="48""#, ""), MM).unwrap();
    assert_eq!((unitless.width, unitless.height), (25.4, 12.7));
    assert!(!unitless.physical_size);
  }

  #[test]
  fn shapes() {
    let s = parse(
      &svg(
        r#"width="100" height="100""#,
        r#"<rect x="1" y="2" width="10" height="20" rx="3"/>
                   <rect width="10" height="20" ry="50"/>
                   <circle cx="50" cy="50" r="10"/>
                   <ellipse cx="50" cy="50" rx="10" ry="5"/>
                   <line x1="1" y1="2" x2="3" y2="4" stroke="black"/>
                   <polyline points="0,0 10,0 10,10"/>
                   <polygon points="0 0 10 0 10 10"/>
                   <path d="M 0 0 A 5 5 0 0 1 10 0 Q 15 5 10 10 C 5 15 0 10 0 0 Z"/>
                   <rect width="0" height="10"/><circle r="-1"/><polyline points="5 5"/>"#,
      ),
      Units {
        px: 1.0,
        inch: 96.0,
      },
    )
    .unwrap();
    assert_eq!(s.shapes.len(), 8);
    // Arcs end exactly on their end points; kurbo's control points may be 1 ulp off.
    let ends = |s: &Shape| {
      let ends: Vec<[f64; 2]> = s
        .segs
        .iter()
        .filter_map(|g| seg_points(g).last().copied())
        .collect();
      ends
        .iter()
        .fold([f64::MAX, f64::MAX, f64::MIN, f64::MIN], |b, p| {
          [
            b[0].min(p[0]),
            b[1].min(p[1]),
            b[2].max(p[0]),
            b[3].max(p[1]),
          ]
        })
    };
    for (i, want) in [
      [1.0, 2.0, 11.0, 22.0],
      [0.0, 0.0, 10.0, 20.0],
      [40.0, 40.0, 60.0, 60.0],
      [40.0, 45.0, 60.0, 55.0],
    ]
    .into_iter()
    .enumerate()
    {
      assert_eq!(ends(&s.shapes[i]), want);
      let b = bbox(&s.shapes[i]);
      assert!(
        b.iter().zip(want).all(|(b, w)| (b - w).abs() < 1e-12),
        "{b:?}"
      );
    }
    assert_eq!(
      s.shapes[4].segs,
      vec![Seg::Move([1.0, 2.0]), Seg::Line([3.0, 4.0])]
    );
    assert_eq!(s.shapes[5].segs.last(), Some(&Seg::Line([10.0, 10.0])));
    assert_eq!(s.shapes[6].segs.last(), Some(&Seg::Close));
    assert!(matches!(s.shapes[7].segs[1], Seg::Cubic(..)));
    assert!(matches!(
      s.shapes[7].segs.iter().rev().nth(2),
      Some(Seg::Quad(..))
    ));
  }

  #[test]
  fn path_data_matches_svgtypes_except_exact_arc_ends() {
    use svgtypes::SimplePathSegment as S;
    let coords = |s: &S| match *s {
      S::MoveTo { x, y } | S::LineTo { x, y } => vec![x, y],
      S::Quadratic { x1, y1, x, y } => vec![x1, y1, x, y],
      S::CurveTo {
        x1,
        y1,
        x2,
        y2,
        x,
        y,
      } => vec![x1, y1, x2, y2, x, y],
      S::ClosePath => vec![],
    };
    for d in [
      "M10 20 l 5 5 h 3 v -2 H 1 V 2 z l 1 1 m 2 2 3 3 Z",
      "M 0 0 c 1 2 3 4 5 6 s 1 1 2 2 S 9 9 10 10 C 1 1 2 2 3 3",
      "m 1 1 q 2 2 4 0 t 4 0 T 12 1 Q 0 0 1 1 t 1 1",
      "M 0 0 a 5 3 30 1 0 10 0 A 5 5 0 0 1 20 0 a 0 5 0 0 1 5 5 L 1 1 ; garbage",
      "M 1e3 -2.5e-3 L .5 .5 l-1-1 1e-7 0",
    ] {
      let ours = path_data(d);
      let theirs: Vec<S> = svgtypes::SimplifyingPathParser::from(d)
        .map_while(Result::ok)
        .collect();
      assert_eq!(ours.len(), theirs.len(), "{d}");
      for (a, b) in ours.iter().zip(&theirs) {
        let (a, b) = (coords(a), coords(b));
        assert_eq!(a.len(), b.len(), "{d}");
        assert!(
          a.iter().zip(&b).all(|(a, b)| (a - b).abs() < 1e-9),
          "{d}: {a:?} {b:?}"
        );
      }
    }
    let arc = path_data("M 0 0 A 7 7 0 0 1 3.3 4.4 a 1 2 45 1 1 -1.1 0.7");
    assert!(
      matches!(arc[arc.len() - 1], S::CurveTo { x, y, .. } if x == 3.3 - 1.1 && y == 4.4 + 0.7)
    );
  }

  #[test]
  fn styles_cascade_and_inherit() {
    let s = parse(
            &svg(
                r#"width="10" height="10""#,
                r##"<style>.cut { stroke: #ff0000; fill: none } #e { stroke: blue !important }
                   g > path:first-child { stroke-width: 0 }</style>
                   <path class="cut" d="M0 0 L1 1"/>
                   <path class="cut" id="e" style="stroke: green" d="M0 0 L1 1"/>
                   <g fill="#00ff00" color="#123456"><path d="M0 0 L1 1"/><path stroke="currentColor" d="M0 0 L1 1"/></g>
                   <g visibility="hidden"><path d="M0 0 L1 1"/><path visibility="visible" d="M0 0 L1 1"/></g>
                   <path fill="none" d="M0 0 L1 1"/>
                   <path display="none" d="M0 0 L1 1"/><g style="display:none"><path d="M0 0 L1 1"/></g>
                   <linearGradient id="lg"><stop offset="0"/></linearGradient><linearGradient id="empty"/>
                   <path fill="url(#lg)" d="M0 0 L1 1"/><path fill="url(#empty) red" d="M0 0 L1 1"/>"##,
            ),
            Units { px: 1.0, inch: 96.0 },
        )
        .unwrap();
    let p: Vec<_> = s
      .shapes
      .iter()
      .map(|s| (s.fill, s.stroke, s.visible))
      .collect();
    use Paint::*;
    assert_eq!(
      p,
      vec![
        (None, Color([255, 0, 0]), true),
        (None, Color([0, 0, 255]), true),
        (Color([0, 255, 0]), None, true),
        (Color([0, 255, 0]), Color([0x12, 0x34, 0x56]), true),
        (Color([0, 0, 0]), None, false),
        (Color([0, 0, 0]), None, true),
        (None, None, false),
        (Server, None, true),
        (Color([255, 0, 0]), None, true),
      ]
    );
  }

  #[test]
  fn use_symbol_nested_svg_and_switch() {
    let s = parse(
            &svg(
                r#"width="100" height="100""#,
                r##"<defs><rect id="r" width="10" height="10"/>
                     <symbol id="sym" viewBox="0 0 1 1"><rect width="1" height="1"/></symbol></defs>
                   <use xlink:href="#r" x="5" y="6" transform="translate(1 1)"/>
                   <use href="#sym" x="20" y="20" width="30" height="30"/>
                   <svg x="50" y="50" width="20" height="20" viewBox="0 0 2 2"><rect width="2" height="2"/></svg>
                   <switch><foreignObject requiredExtensions="x"/><g systemLanguage="de"><rect width="99" height="99"/></g>
                     <g id="layer-A"><rect width="3" height="4"/></g><rect width="5" height="5"/></switch>
                   <g id="loop"><use href="#loop"/></g><use href="#missing"/>"##,
            ),
            Units { px: 1.0, inch: 96.0 },
        )
        .unwrap();
    let b: Vec<_> = s.shapes.iter().map(bbox).collect();
    assert_eq!(
      b,
      vec![
        [6.0, 7.0, 16.0, 17.0],
        [20.0, 20.0, 50.0, 50.0],
        [50.0, 50.0, 70.0, 70.0],
        [0.0, 0.0, 3.0, 4.0],
      ]
    );
    assert_eq!(s.shapes[3].groups, vec!["layer-A".to_string()]);
  }

  #[test]
  fn hostile_input_is_bounded() {
    // Nine levels of ten `<use>` each would be 10^9 rectangles.
    let mut body = String::from(r#"<rect id="u0" width="1" height="1"/>"#);
    for i in 1..10 {
      body.push_str(&format!(r#"<g id="u{i}">"#));
      for _ in 0..10 {
        body.push_str(&format!(r##"<use href="#u{}"/>"##, i - 1));
      }
      body.push_str("</g>");
    }
    let err = parse(&svg(r#"width="10" height="10""#, &body), MM).unwrap_err();
    assert!(err.contains("elements"), "{err}");
    let deep = format!("{}{}", "<g>".repeat(300), "</g>".repeat(300));
    assert!(parse(&svg(r#"width="10" height="10""#, &deep), MM).is_err());
    assert!(parse("<html/>", MM).is_err());
    assert!(parse(&svg(r#"width="0" height="10""#, ""), MM).is_err());
  }
}
