//! LightBurn file parsing and generation

use roxmltree::{Document, Node};
use quick_xml::{Writer, events::{Event, BytesStart, BytesEnd, BytesText}};
use std::io::Cursor;

use crate::types::{
    ConversionError, LightburnFile, Shape, ShapeType, 
    Transform, Point, Path, PathCommand, CutSetting,
    SvgFile,
};

/// Parse LightBurn file content into our internal representation
pub fn parse(content: &str) -> Result<LightburnFile, ConversionError> {
    let doc = Document::parse(content)
        .map_err(|e| ConversionError::XmlParseError(e.to_string()))?;
    
    let root = doc.root_element();
    if !root.has_tag_name("LightBurnProject") {
        return Err(ConversionError::XmlParseError("Root element is not <LightBurnProject>".to_string()));
    }
    
    // Extract basic project info
    let app_version = root.attribute("AppVersion").unwrap_or("unknown");
    let device_name = root.attribute("DeviceName").unwrap_or("unknown");
    let format_version = root.attribute("FormatVersion").unwrap_or("0");
    let material_height = root.attribute("MaterialHeight")
        .unwrap_or("0").parse::<f64>().unwrap_or(0.0);
    let mirror_x = root.attribute("MirrorX").unwrap_or("False") == "True";
    let mirror_y = root.attribute("MirrorY").unwrap_or("False") == "True";
    
    // Parse thumbnail if present
    let thumbnail = root.children()
        .find(|n| n.has_tag_name("Thumbnail"))
        .and_then(|n| n.attribute("Source"))
        .map(|s| s.to_string());
    
    // Parse cut settings
    let cut_settings = root.children()
        .filter(|n| n.has_tag_name("CutSetting"))
        .map(parse_cut_setting)
        .collect::<Vec<_>>();
    
    // Parse shapes
    let shapes = root.children()
        .filter(|n| n.is_element() && n.has_tag_name("Shape"))
        .map(parse_shape)
        .collect::<Result<Vec<_>, _>>()?;
    
    Ok(LightburnFile {
        version: format_version.to_string(),
        app_version: app_version.to_string(),
        device_name: device_name.to_string(),
        material_height,
        mirror_x,
        mirror_y,
        thumbnail,
        cut_settings,
        shapes,
    })
}

/// Parse a cut setting node
fn parse_cut_setting(node: Node) -> CutSetting {
    let mut properties = std::collections::HashMap::new();
    
    // Extract index, name, power and speed
    let index = node.attribute("index")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    
    let name = node.attribute("name")
        .unwrap_or("").to_string();
    
    let min_power = node.children()
        .find(|n| n.has_tag_name("minPower"))
        .and_then(|n| n.attribute("Value"))
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    
    let max_power = node.children()
        .find(|n| n.has_tag_name("maxPower"))
        .and_then(|n| n.attribute("Value"))
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    
    let speed = node.children()
        .find(|n| n.has_tag_name("speed"))
        .and_then(|n| n.attribute("Value"))
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    
    // Store any other properties
    for child in node.children().filter(|n| n.is_element()) {
        if let Some(value) = child.attribute("Value") {
            properties.insert(child.tag_name().name().to_string(), value.to_string());
        }
    }
    
    CutSetting {
        index,
        name,
        min_power,
        max_power,
        speed,
        properties,
    }
}

/// Parse a shape node recursively
fn parse_shape(node: Node) -> Result<Shape, ConversionError> {
    let shape_type = node.attribute("Type")
        .unwrap_or("").to_string();
    
    let cut_index = node.attribute("CutIndex")
        .and_then(|s| s.parse::<usize>().ok());
    
    // Parse transform if present
    let transform = node.children()
        .find(|n| n.has_tag_name("XForm"))
        .and_then(|n| n.text())
        .map(parse_transform);
    
    let shape_type = match shape_type.as_str() {
        "Rect" => ShapeType::Rect,
        "Ellipse" => ShapeType::Ellipse,
        "Path" => ShapeType::Path,
        "Group" => ShapeType::Group,
        _ => ShapeType::Other(shape_type.clone()),
    };
    
    // Create a base shape with the common properties
    let mut shape = Shape {
        shape_type: shape_type.clone(),
        cut_index,
        transform,
        width: None,
        height: None,
        corner_radius: None,
        rx: None,
        ry: None,
        path: None,
        children: None,
        style: None,
        fill: None,
        stroke: None,
        stroke_width: None,
    };
    
    // Fill in shape-specific properties
    match shape_type {
        ShapeType::Rect => {
            shape.width = node.attribute("W").and_then(|s| s.parse::<f64>().ok());
            shape.height = node.attribute("H").and_then(|s| s.parse::<f64>().ok());
            shape.corner_radius = node.attribute("Cr").and_then(|s| s.parse::<f64>().ok());
        },
        ShapeType::Ellipse => {
            shape.rx = node.attribute("Rx").and_then(|s| s.parse::<f64>().ok());
            shape.ry = node.attribute("Ry").and_then(|s| s.parse::<f64>().ok());
        },
        ShapeType::Path => {
            // Parse path data from VertList and PrimList
            let vert_list = node.children()
                .find(|n| n.has_tag_name("VertList"))
                .map(|n| n.text().unwrap_or(""))
                .unwrap_or("");
            
            let prim_list = node.children()
                .find(|n| n.has_tag_name("PrimList"))
                .map(|n| n.text().unwrap_or(""))
                .unwrap_or("");
            
            shape.path = Some(parse_path_data(vert_list, prim_list)?);
        },
        ShapeType::Group => {
            // Parse children recursively
            let children = node.children()
                .filter(|n| n.is_element() && n.has_tag_name("Children"))
                .flat_map(|n| n.children().filter(|c| c.is_element() && c.has_tag_name("Shape")))
                .map(parse_shape)
                .collect::<Result<Vec<_>, _>>()?;
            
            shape.children = Some(children);
        },
        ShapeType::Other(_) => {
            // Just keep the basic shape info
        }
    }
    
    Ok(shape)
}

/// Parse transformation matrix from a space-separated string
fn parse_transform(transform_str: &str) -> Transform {
    let parts: Vec<f64> = transform_str.split_whitespace()
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();
    
    if parts.len() >= 6 {
        Transform {
            a: parts[0],
            b: parts[1],
            c: parts[2],
            d: parts[3],
            e: parts[4],
            f: parts[5],
        }
    } else {
        Transform::default()
    }
}

/// Parse path data from VertList and PrimList
fn parse_path_data(vert_list: &str, prim_list: &str) -> Result<Path, ConversionError> {
    let mut points = Vec::new();
    
    // Parse vertices
    for vert in vert_list.split('V').filter(|s| !s.is_empty()) {
        let parts: Vec<&str> = vert.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        
        let x = parts[0].parse::<f64>().map_err(|_| {
            ConversionError::InvalidShapeData(format!("Invalid vertex x: {}", parts[0]))
        })?;
        
        let y = parts[1].parse::<f64>().map_err(|_| {
            ConversionError::InvalidShapeData(format!("Invalid vertex y: {}", parts[1]))
        })?;
        
        let mut point = Point { x, y, c0x: None, c0y: None, c1x: None, c1y: None };
        
        // Parse control points if present
        for part in &parts[2..] {
            if part.starts_with("c0x") {
                point.c0x = part[3..].parse::<f64>().ok();
            } else if part.starts_with("c0y") {
                point.c0y = part[3..].parse::<f64>().ok();
            } else if part.starts_with("c1x") {
                point.c1x = part[3..].parse::<f64>().ok();
            } else if part.starts_with("c1y") {
                point.c1y = part[3..].parse::<f64>().ok();
            }
        }
        
        points.push(point);
    }
    
    // Parse commands
    let mut commands = Vec::new();
    for prim in prim_list.split_whitespace() {
        if prim.len() < 2 {
            continue;
        }
        
        let cmd_type = &prim[0..1];
        let args = &prim[1..];
        
        match cmd_type {
            "L" => {
                if let Some((p0, p1)) = args.split_once(' ') {
                    if let (Ok(p0), Ok(p1)) = (p0.parse::<usize>(), p1.parse::<usize>()) {
                        commands.push(PathCommand::LineTo(p0, p1));
                    }
                }
            },
            "B" => {
                if let Some((p0, p1)) = args.split_once(' ') {
                    if let (Ok(p0), Ok(p1)) = (p0.parse::<usize>(), p1.parse::<usize>()) {
                        commands.push(PathCommand::BezierTo(p0, p1));
                    }
                }
            },
            "M" => {
                if let Some((p0, p1)) = args.split_once(' ') {
                    if let (Ok(p0), Ok(p1)) = (p0.parse::<usize>(), p1.parse::<usize>()) {
                        commands.push(PathCommand::MoveTo(p0, p1));
                    }
                }
            },
            "Z" => {
                commands.push(PathCommand::Close);
            },
            _ => {}
        }
    }
    
    Ok(Path { points, commands })
}

/// Generate LightBurn v1 format
pub fn generate_v1(svg_file: &SvgFile) -> Result<String, ConversionError> {
    let buffer = Cursor::new(Vec::new());
    let mut writer = Writer::new_with_indent(buffer, b' ', 4);
    
    // Start XML document
    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Create root element
    let mut elem = BytesStart::new("LightBurnProject");
    elem.push_attribute(("AppVersion", "1.7.08"));
    elem.push_attribute(("DeviceName", "GRBL-PicoCNC"));
    elem.push_attribute(("FormatVersion", "0"));
    elem.push_attribute(("MaterialHeight", "0"));
    elem.push_attribute(("MirrorX", "False"));
    elem.push_attribute(("MirrorY", "False"));
    
    writer.write_event(Event::Start(elem))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add thumbnail placeholder
    let mut thumb_elem = BytesStart::new("Thumbnail");
    thumb_elem.push_attribute(("Source", ""));
    writer.write_event(Event::Empty(thumb_elem))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add variable text placeholder
    writer.write_event(Event::Start(BytesStart::new("VariableText")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    let elements = [
        ("Start", "0"),
        ("End", "999"),
        ("Current", "0"),
        ("Increment", "1"),
        ("AutoAdvance", "0"),
    ];
    
    for (name, value) in elements {
        let mut elem = BytesStart::new(name);
        elem.push_attribute(("Value", value));
        writer.write_event(Event::Empty(elem))
            .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    }
    
    writer.write_event(Event::End(BytesEnd::new("VariableText")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add default UI preferences
    writer.write_event(Event::Start(BytesStart::new("UIPrefs")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add some standard UI preferences
    let ui_prefs = [
        ("Optimize_ByLayer", "0"),
        ("Optimize_ByGroup", "-1"),
        ("Optimize_ByPriority", "1"),
        ("Optimize_WhichDirection", "0"),
        ("Optimize_InnerToOuter", "1"),
        ("Optimize_ByDirection", "0"),
        ("Optimize_ReduceTravel", "1"),
        ("Optimize_HideBacklash", "0"),
        ("Optimize_ReduceDirChanges", "0"),
        ("Optimize_ChooseCorners", "0"),
        ("Optimize_AllowReverse", "1"),
        ("Optimize_RemoveOverlaps", "0"),
        ("Optimize_OptimalEntryPoint", "1"),
        ("Optimize_OverlapDist", "0.025"),
    ];
    
    for (name, value) in ui_prefs {
        let mut elem = BytesStart::new(name);
        elem.push_attribute(("Value", value));
        writer.write_event(Event::Empty(elem))
            .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    }
    
    writer.write_event(Event::End(BytesEnd::new("UIPrefs")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add default cut setting
    let mut cut_elem = BytesStart::new("CutSetting");
    cut_elem.push_attribute(("type", "Cut"));
    writer.write_event(Event::Start(cut_elem))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add standard cut setting values
    let cut_settings = [
        ("index", "0"),
        ("name", "Schneiden"),
        ("minPower", "17.5"),
        ("maxPower", "90"),
        ("maxPower2", "20"),
        ("speed", "4.16667"),
        ("angle", "90"),
        ("priority", "0"),
        ("tabCount", "1"),
        ("tabCountMax", "1"),
        ("tabSpacing", "50.04"),
    ];
    
    for (name, value) in cut_settings {
        let mut elem = BytesStart::new(name);
        elem.push_attribute(("Value", value));
        writer.write_event(Event::Empty(elem))
            .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    }
    
    writer.write_event(Event::End(BytesEnd::new("CutSetting")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Write shapes
    let mut group_elem = BytesStart::new("Shape");
    group_elem.push_attribute(("Type", "Group"));
    writer.write_event(Event::Start(group_elem))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Default transform
    let xform_elem = BytesStart::new("XForm");
    writer.write_event(Event::Start(xform_elem))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    writer.write_event(Event::Text(BytesText::new("1 0 0 1 0 0")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    writer.write_event(Event::End(BytesEnd::new("XForm")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add children container
    writer.write_event(Event::Start(BytesStart::new("Children")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Write shapes
    for shape in &svg_file.shapes {
        write_shape(&mut writer, shape, 0)?;
    }
    
    // Close children container
    writer.write_event(Event::End(BytesEnd::new("Children")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Close group
    writer.write_event(Event::End(BytesEnd::new("Shape")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add empty notes
    let mut notes_elem = BytesStart::new("Notes");
    notes_elem.push_attribute(("ShowOnLoad", "0"));
    notes_elem.push_attribute(("Notes", ""));
    writer.write_event(Event::Empty(notes_elem))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // End root element
    writer.write_event(Event::End(BytesEnd::new("LightBurnProject")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    String::from_utf8(writer.into_inner().into_inner())
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))
}

/// Generate LightBurn v2 format
pub fn generate_v2(svg_file: &SvgFile) -> Result<String, ConversionError> {
    let buffer = Cursor::new(Vec::new());
    let mut writer = Writer::new_with_indent(buffer, b' ', 4);
    
    // Start XML document
    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Create root element - only difference is FormatVersion="1"
    let mut elem = BytesStart::new("LightBurnProject");
    elem.push_attribute(("AppVersion", "1.7.08"));
    elem.push_attribute(("DeviceName", "GRBL-PicoCNC"));
    elem.push_attribute(("FormatVersion", "1"));  // Version 2 format
    elem.push_attribute(("MaterialHeight", "0"));
    elem.push_attribute(("MirrorX", "False"));
    elem.push_attribute(("MirrorY", "False"));
    
    writer.write_event(Event::Start(elem))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add thumbnail placeholder
    let mut thumb_elem = BytesStart::new("Thumbnail");
    thumb_elem.push_attribute(("Source", ""));
    writer.write_event(Event::Empty(thumb_elem))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add variable text placeholder
    writer.write_event(Event::Start(BytesStart::new("VariableText")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    let elements = [
        ("Start", "0"),
        ("End", "999"),
        ("Current", "0"),
        ("Increment", "1"),
        ("AutoAdvance", "0"),
    ];
    
    for (name, value) in elements {
        let mut elem = BytesStart::new(name);
        elem.push_attribute(("Value", value));
        writer.write_event(Event::Empty(elem))
            .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    }
    
    writer.write_event(Event::End(BytesEnd::new("VariableText")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add default UI preferences
    writer.write_event(Event::Start(BytesStart::new("UIPrefs")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add some standard UI preferences
    let ui_prefs = [
        ("Optimize_ByLayer", "0"),
        ("Optimize_ByGroup", "-1"),
        ("Optimize_ByPriority", "1"),
        ("Optimize_WhichDirection", "0"),
        ("Optimize_InnerToOuter", "1"),
        ("Optimize_ByDirection", "0"),
        ("Optimize_ReduceTravel", "1"),
        ("Optimize_HideBacklash", "0"),
        ("Optimize_ReduceDirChanges", "0"),
        ("Optimize_ChooseCorners", "0"),
        ("Optimize_AllowReverse", "1"),
        ("Optimize_RemoveOverlaps", "0"),
        ("Optimize_OptimalEntryPoint", "1"),
        ("Optimize_OverlapDist", "0.025"),
    ];
    
    for (name, value) in ui_prefs {
        let mut elem = BytesStart::new(name);
        elem.push_attribute(("Value", value));
        writer.write_event(Event::Empty(elem))
            .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    }
    
    writer.write_event(Event::End(BytesEnd::new("UIPrefs")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add default cut setting
    let mut cut_elem = BytesStart::new("CutSetting");
    cut_elem.push_attribute(("type", "Cut"));
    writer.write_event(Event::Start(cut_elem))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add standard cut setting values
    let cut_settings = [
        ("index", "0"),
        ("name", "Schneiden"),
        ("minPower", "17.5"),
        ("maxPower", "90"),
        ("maxPower2", "20"),
        ("speed", "4.16667"),
        ("angle", "90"),
        ("priority", "0"),
        ("tabCount", "1"),
        ("tabCountMax", "1"),
        ("tabSpacing", "50.04"),
    ];
    
    for (name, value) in cut_settings {
        let mut elem = BytesStart::new(name);
        elem.push_attribute(("Value", value));
        writer.write_event(Event::Empty(elem))
            .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    }
    
    writer.write_event(Event::End(BytesEnd::new("CutSetting")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Write shapes
    let mut group_elem = BytesStart::new("Shape");
    group_elem.push_attribute(("Type", "Group"));
    writer.write_event(Event::Start(group_elem))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Default transform
    let xform_elem = BytesStart::new("XForm");
    writer.write_event(Event::Start(xform_elem))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    writer.write_event(Event::Text(BytesText::new("1 0 0 1 0 0")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    writer.write_event(Event::End(BytesEnd::new("XForm")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add children container
    writer.write_event(Event::Start(BytesStart::new("Children")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Write shapes
    for shape in &svg_file.shapes {
        write_shape(&mut writer, shape, 0)?;
    }
    
    // Close children container
    writer.write_event(Event::End(BytesEnd::new("Children")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Close group
    writer.write_event(Event::End(BytesEnd::new("Shape")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // Add empty notes
    let mut notes_elem = BytesStart::new("Notes");
    notes_elem.push_attribute(("ShowOnLoad", "0"));
    notes_elem.push_attribute(("Notes", ""));
    writer.write_event(Event::Empty(notes_elem))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    // End root element
    writer.write_event(Event::End(BytesEnd::new("LightBurnProject")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    String::from_utf8(writer.into_inner().into_inner())
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))
}

/// Write a shape to the XML writer
fn write_shape<W: std::io::Write>(writer: &mut Writer<W>, shape: &Shape, cut_index: usize) -> Result<(), ConversionError> {
    let mut elem = BytesStart::new("Shape");
    
    match &shape.shape_type {        ShapeType::Rect => {
            elem.push_attribute(("Type", "Rect"));
            elem.push_attribute(("CutIndex", cut_index.to_string().as_str()));
            
            writer.write_event(Event::Start(elem))
                .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
            
            if let Some(width) = shape.width {
                elem = BytesStart::new("W");
                elem.push_attribute(("Value", width.to_string().as_str()));
                writer.write_event(Event::Empty(elem))
                    .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
            }
            
            if let Some(height) = shape.height {
                elem = BytesStart::new("H");
                elem.push_attribute(("Value", height.to_string().as_str()));
                writer.write_event(Event::Empty(elem))
                    .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
            }
            
            if let Some(cr) = shape.corner_radius {
                elem = BytesStart::new("Cr");
                elem.push_attribute(("Value", cr.to_string().as_str()));
                writer.write_event(Event::Empty(elem))
                    .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
            }
            
            write_transform(writer, shape)?;
        },        ShapeType::Ellipse => {
            elem.push_attribute(("Type", "Ellipse"));
            elem.push_attribute(("CutIndex", cut_index.to_string().as_str()));
            
            writer.write_event(Event::Start(elem))
                .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
            
            if let Some(rx) = shape.rx {
                elem = BytesStart::new("Rx");
                elem.push_attribute(("Value", rx.to_string().as_str()));
                writer.write_event(Event::Empty(elem))
                    .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
            }
            
            if let Some(ry) = shape.ry {
                elem = BytesStart::new("Ry");
                elem.push_attribute(("Value", ry.to_string().as_str()));
                writer.write_event(Event::Empty(elem))
                    .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
            }
            
            write_transform(writer, shape)?;
        },        ShapeType::Path => {
            elem.push_attribute(("Type", "Path"));
            elem.push_attribute(("CutIndex", cut_index.to_string().as_str()));
            
            writer.write_event(Event::Start(elem))
                .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
            
            write_transform(writer, shape)?;
            
            if let Some(path) = &shape.path {
                if !path.points.is_empty() {
                    // Write vertices
                    writer.write_event(Event::Start(BytesStart::new("VertList")))
                        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
                    
                    let mut vert_text = String::new();
                    for point in &path.points {
                        vert_text.push_str(&format!("V{} {} ", point.x, point.y));
                        if let Some(c0x) = point.c0x {
                            vert_text.push_str(&format!("c0x{} c0y{} ", c0x, point.c0y.unwrap_or(0.0)));
                        }
                        if let Some(c1x) = point.c1x {
                            vert_text.push_str(&format!("c1x{} c1y{} ", c1x, point.c1y.unwrap_or(0.0)));
                        }
                    }
                    
                    writer.write_event(Event::Text(BytesText::new(&vert_text)))
                        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
                    
                    writer.write_event(Event::End(BytesEnd::new("VertList")))
                        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
                    
                    // Write commands
                    writer.write_event(Event::Start(BytesStart::new("PrimList")))
                        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
                    
                    let mut prim_text = String::new();
                    for cmd in &path.commands {
                        match cmd {
                            PathCommand::MoveTo(p0, p1) => {
                                prim_text.push_str(&format!("M{} {} ", p0, p1));
                            },
                            PathCommand::LineTo(p0, p1) => {
                                prim_text.push_str(&format!("L{} {} ", p0, p1));
                            },
                            PathCommand::BezierTo(p0, p1) => {
                                prim_text.push_str(&format!("B{} {} ", p0, p1));
                            },
                            PathCommand::Close => {
                                prim_text.push_str("Z ");
                            },
                        }
                    }
                    
                    writer.write_event(Event::Text(BytesText::new(&prim_text)))
                        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
                    
                    writer.write_event(Event::End(BytesEnd::new("PrimList")))
                        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
                }
            }
        },
        ShapeType::Group => {
            elem.push_attribute(("Type", "Group"));
            
            writer.write_event(Event::Start(elem))
                .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
            
            write_transform(writer, shape)?;
            
            if let Some(children) = &shape.children {
                writer.write_event(Event::Start(BytesStart::new("Children")))
                    .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
                
                for child in children {
                    write_shape(writer, child, cut_index)?;
                }
                
                writer.write_event(Event::End(BytesEnd::new("Children")))
                    .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
            }
        },
        ShapeType::Other(name) => {
            elem.push_attribute(("Type", name.as_str()));
            
            writer.write_event(Event::Start(elem))
                .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
        }
    }
    
    writer.write_event(Event::End(BytesEnd::new("Shape")))
        .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    
    Ok(())
}

// Helper to write transform element
fn write_transform<W: std::io::Write>(writer: &mut Writer<W>, shape: &Shape) -> Result<(), ConversionError> {
    if let Some(transform) = &shape.transform {
        writer.write_event(Event::Start(BytesStart::new("XForm")))
            .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
        
        let transform_str = format!(
            "{} {} {} {} {} {}", 
            transform.a, transform.b, transform.c, transform.d, transform.e, transform.f
        );
        
        writer.write_event(Event::Text(BytesText::new(&transform_str)))
            .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
        
        writer.write_event(Event::End(BytesEnd::new("XForm")))
            .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    } else {
        // Default transform
        writer.write_event(Event::Start(BytesStart::new("XForm")))
            .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
        
        writer.write_event(Event::Text(BytesText::new("1 0 0 1 0 0")))
            .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
        
        writer.write_event(Event::End(BytesEnd::new("XForm")))
            .map_err(|e| ConversionError::XmlGenerateError(e.to_string()))?;
    }
    
    Ok(())
}
