# @deepnest/nofitpolygon

NAPI-RS bindings for calculating No-Fit Polygons (NFP) using the `babushka` Rust crate. This package exposes functions to compute the NFP of two polygons, which is a core operation in nesting algorithms.

## Installation

```bash
npm install @deepnest/nofitpolygon
# or
yarn add @deepnest/nofitpolygon
```

## Usage

```javascript
const { noFitPolygon } = require('@deepnest/nofitpolygon');

const polygonAWithHole = {
  outer: [ // Outer boundary of A
    { x: 0, y: 0 },
    { x: 200, y: 0 },
    { x: 200, y: 150 },
    { x: 0, y: 150 },
  ],
  inner: [ // Optional list of inner holes for A
    [ // First hole
      { x: 50, y: 50 },
      { x: 150, y: 50 },
      { x: 150, y: 100 },
      { x: 50, y: 100 },
    ]
  ]
};

const polygonB = { // Orbiting polygon (its own holes are ignored in this NFP calculation)
  outer: [
    { x: 0, y: 0 },
    { x: 30, y: 0 },
    { x: 15, y: 20 },
  ]
  // inner: [] // Optional, but not used for polygonB by babushka's MultiPolygon NFP
};

try {
  // Calculate NFP for polygonB around polygonA's outer boundary
  // and NFP for polygonB trying to fit into polygonA's holes (from the inside of the hole)
  const nfpPaths = noFitPolygon(polygonAWithHole, polygonB, true, true); 
                                    // includeOuterNfp = true, includeHolesNfp = true

  if (nfpPaths.length > 0) {
    console.log('NFP paths calculated:');
    nfpPaths.forEach((path, i) => {
      console.log(`  Path ${i}:`);
      path.forEach(point => console.log(`    { x: ${point.x}, y: ${point.y} }`));
    });
  } else {
    console.log('No NFP paths were generated for the given configuration.');
  }

} catch (error) {
  console.error('Error calculating NFP:', error);
}
```

### API

#### `noFitPolygon(polygonA: Polygon, polygonB: Polygon, includeOuterNfp: boolean, includeHolesNfp: boolean): PolygonPath[]`

Computes the No-Fit Polygon.

*   `polygonA: Polygon`: The stationary polygon. This polygon can have holes.
    *   `outer: Point[]`: An array of points `{ x: number, y: number }` defining the outer boundary.
    *   `inner?: Point[][]`: An optional array of paths (each path being `Point[]`) defining the inner hole boundaries.
*   `polygonB: Polygon`: The orbiting polygon. For the NFP calculation with `MultiPolygon` in `babushka`, effectively only its `outer` boundary is used. Its `inner` holes, if provided, do not influence the NFP result with `polygonA`.
    *   `outer: Point[]`: An array of points defining its outer boundary.
    *   `inner?: Point[][]`: Optional.
*   `includeOuterNfp: boolean`:
    *   If `true`, computes the NFP of `polygonA`'s outer boundary against `polygonB`'s outer boundary. This is typically used to see where `polygonB` can be placed *around* `polygonA` or just touching its exterior.
*   `includeHolesNfp: boolean`:
    *   If `true`, computes the NFPs of `polygonA`'s inner holes against `polygonB`'s outer boundary. These NFPs define where `polygonB` can be placed if it were to fit *inside* one of `polygonA`'s holes (from the hole's perspective, it's like an "outer" NFP for the hole).

*   **Returns**: `PolygonPath[]`
    *   An array of `PolygonPath`s, where each `PolygonPath` is an array of `Point`s.
    *   Returns an empty array (`[]`) if no NFP paths are generated for the given configuration (e.g., if the polygons cannot touch appropriately or due to geometric degeneracies).

Where:
*   `PolygonPath = Point[]`
*   `Point = { x: number, y: number }`

## Building from Source

If you need to build the package from source (e.g., for a platform not covered by prebuilt binaries):

1.  Ensure you have Rust and Node.js installed.
2.  Install `@napi-rs/cli`: `npm install -g @napi-rs/cli` or `yarn global add @napi-rs/cli`.
3.  Clone the repository and navigate to this package's directory:
    ```bash
    # git clone ... (clone the main deepnest repo)
    cd crates/npm/deepnest-nofitpolygon
    ```
4.  Install dependencies and build:
    ```bash
    yarn install # or npm install
    yarn build # or npm run build
    ```

## Development

Run tests:
```bash
yarn test # or npm run test
```

Build in debug mode:
```bash
yarn build:debug # or npm run build:debug
```

## License

MIT (or match the main project license)
```
