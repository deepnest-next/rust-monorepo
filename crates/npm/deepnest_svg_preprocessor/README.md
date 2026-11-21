# svg-preprocessor

This nodejs lib, is build in rust and can be used with nodejs and electron.

`loadSvgString`/`loadSvgFile` normalize the SVG so that deepnest's parser can handle it: CSS,
`<use>`, units, `viewBox` and nested transforms are resolved, and every shape becomes one
`<path>` with absolute coordinates. All numbers stay `f64` (`src/svg_f64.rs`, on roxmltree +
svgtypes + simplecss): usvg, used before, stores `f32`, so a 1000 mm box came back 2834.6457
instead of 2834.645669291339 units wide at 72 units per inch.

