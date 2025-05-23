const test = require('ava');
const { noFitPolygon } = require('../index.js');

test('NFP for two simple squares (B outside A)', (t) => {
  const squareA = { // Stationary
    outer: [ { x: 0, y: 0 }, { x: 4, y: 0 }, { x: 4, y: 4 }, { x: 0, y: 4 } ],
  };
  const squareB = { // Orbiting
    outer: [ { x: 0, y: 0 }, { x: 1, y: 0 }, { x: 1, y: 1 }, { x: 0, y: 1 } ],
  };

  try {
    // NFP of A's outer boundary vs B's outer boundary.
    // Should allow B to be placed *around* A.
    const result = noFitPolygon(squareA, squareB, true, false); 
                        // includeOuterNfp = true, includeHolesNfp = false

    t.truthy(result, 'Result should not be null');
    t.true(Array.isArray(result), 'Result should be an array of paths');
    t.is(result.length, 1, 'Should return a single NFP path for A_outer vs B_outer');
    
    const path = result[0];
    t.true(Array.isArray(path), 'Path should be an array of points');
    t.true(path.length > 2, 'Path should have at least 3 vertices');
    console.log("NFP Result (B outside A):", JSON.stringify(result, null, 2));
    t.pass();
  } catch (e) {
    t.fail(`NFP calculation failed: ${e.message}`);
  }
});

test('NFP for two simple squares (B inside A, should be empty or specific shape)', (t) => {
  const squareA = { // Stationary, larger
    outer: [ { x: 0, y: 0 }, { x: 10, y: 0 }, { x: 10, y: 10 }, { x: 0, y: 10 } ],
  };
  const squareB = { // Orbiting, smaller
    outer: [ { x: 0, y: 0 }, { x: 2, y: 0 }, { x: 2, y: 2 }, { x: 0, y: 2 } ],
  };

  try {
    // NFP of A's outer boundary vs B's outer boundary, for B to fit *inside* A.
    // Babushka's `Polygon::no_fit_polygon` with `inside=true` (which MultiPolygon wraps) calculates this.
    // This is one way to get the "inner NFP".
    // We are testing the NFP of A's outer boundary, but expecting a result that allows B to be inside A.
    // The MultiPolygon's `no_fit_polygon` for outer boundary uses `inside=false` for its internal call.
    // To get the "true" inner NFP (where B can be placed within A), one would typically use
    // the `Polygon.no_fit_polygon(other, true, search_edges)` if not using MultiPolygon.
    // With MultiPolygon, if A has no holes, and we want B inside A, we'd still use includeOuterNfp=true,
    // and the resulting NFP would be the path B traces *inside* A.
    const result = noFitPolygon(squareA, squareB, true, false); 
                        // includeOuterNfp = true (for A's shell), includeHolesNfp = false

    t.truthy(result, 'Result should not be null for B inside A');
    t.true(Array.isArray(result), 'Result should be an array of paths');
    t.is(result.length, 1, 'Should return a single NFP path for B inside A shell');

    const path = result[0];
    t.true(Array.isArray(path), 'Path should be an array of points');
    t.true(path.length > 2, 'Path should have at least 3 vertices');
    console.log("NFP Result (B inside A's shell):", JSON.stringify(result, null, 2));
    t.pass();
  } catch (e) {
    t.fail(`NFP (B inside A) calculation failed: ${e.message}`);
  }
});

test('NFP should be empty if B cannot fit inside A (B larger than A)', (t) => {
  const squareA = { // Stationary, smaller
    outer: [ { x: 0, y: 0 }, { x: 2, y: 0 }, { x: 2, y: 2 }, { x: 0, y: 2 } ],
  };
  const squareB = { // Orbiting, larger
    outer: [ { x: 0, y: 0 }, { x: 10, y: 0 }, { x: 10, y: 10 }, { x: 0, y: 10 } ],
  };

  try {
    const result = noFitPolygon(squareA, squareB, true, false); // NFP for A's outer boundary
    t.true(Array.isArray(result), 'Result should be an array, possibly empty');
    // Babushka might return an empty list or a list with empty paths if no NFP is formed.
    // The wrapper filters out empty paths, so result should be [].
    t.is(result.length, 0, `Expected empty array for impossible NFP, got: ${JSON.stringify(result)}`);
    t.pass();
  } catch (e) {
    t.fail(`NFP calculation failed unexpectedly: ${e.message}`);
  }
});

test('NFP with a hole (stationary A has hole, orbiting B is simple square)', (t) => {
  const polyAWithHole = { // Stationary L-shape like polygon with a central hole
    outer: [ { x: 0, y: 0 }, { x: 200, y: 0 }, { x: 200, y: 150 }, { x: 0, y: 150 } ],
    inner: [ // One hole
      [ { x: 50, y: 50 }, { x: 150, y: 50 }, { x: 150, y: 100 }, { x: 50, y: 100 } ]
    ]
  };
  const polyB = { // Orbiting small square
    outer: [ { x: 0, y: 0 }, { x: 20, y: 0 }, { x: 20, y: 20 }, { x: 0, y: 20 } ]
  };

  try {
    // Get NFP for A's outer boundary AND NFP for A's hole boundary, both against B.
    const result = noFitPolygon(polyAWithHole, polyB, true, true); 
                        // includeOuterNfp = true, includeHolesNfp = true

    t.truthy(result, 'Result should not be null for hole example');
    t.true(Array.isArray(result), 'Result should be an array of paths');
    // Expecting 2 paths: 1 for outer boundary of A, 1 for the hole of A.
    t.is(result.length, 2, 'Should return 2 NFP paths (one for outer, one for hole)'); 

    console.log("NFP Result (polygon with hole):", JSON.stringify(result, null, 2));
    result.forEach((path, i) => {
      t.true(Array.isArray(path), `Path ${i} should be an array of points`);
      t.true(path.length > 2, `Path ${i} should have at least 3 vertices`);
    });
    t.pass();
  } catch (e) {
    t.fail(`NFP calculation failed for polygon with hole: ${e.message}`);
  }
});

test('NFP with a hole, only outer NFP requested', (t) => {
  const polyAWithHole = {
    outer: [ { x: 0, y: 0 }, { x: 200, y: 0 }, { x: 200, y: 150 }, { x: 0, y: 150 } ],
    inner: [ [ { x: 50, y: 50 }, { x: 150, y: 50 }, { x: 150, y: 100 }, { x: 50, y: 100 } ] ]
  };
  const polyB = {
    outer: [ { x: 0, y: 0 }, { x: 20, y: 0 }, { x: 20, y: 20 }, { x: 0, y: 20 } ]
  };

  try {
    const result = noFitPolygon(polyAWithHole, polyB, true, false); // Outer NFP only
                        // includeOuterNfp = true, includeHolesNfp = false
    t.is(result.length, 1, 'Should return 1 NFP path (outer only)');
    console.log("NFP Result (hole, outer NFP only):", JSON.stringify(result, null, 2));
    t.pass();
  } catch (e) {
    t.fail(`NFP calculation failed: ${e.message}`);
  }
});

test('NFP with a hole, only hole NFP requested', (t) => {
  const polyAWithHole = {
    outer: [ { x: 0, y: 0 }, { x: 200, y: 0 }, { x: 200, y: 150 }, { x: 0, y: 150 } ],
    inner: [ [ { x: 50, y: 50 }, { x: 150, y: 50 }, { x: 150, y: 100 }, { x: 50, y: 100 } ] ]
  };
  const polyB = {
    outer: [ { x: 0, y: 0 }, { x: 20, y: 0 }, { x: 20, y: 20 }, { x: 0, y: 20 } ]
  };

  try {
    const result = noFitPolygon(polyAWithHole, polyB, false, true); // Hole NFP(s) only
                        // includeOuterNfp = false, includeHolesNfp = true
    t.is(result.length, 1, 'Should return 1 NFP path (hole only)');
    console.log("NFP Result (hole, hole NFP only):", JSON.stringify(result, null, 2));
    t.pass();
  } catch (e) {
    t.fail(`NFP calculation failed: ${e.message}`);
  }
});

test('NFP with a hole, no NFPs requested (should be empty array)', (t) => {
  const polyAWithHole = {
    outer: [ { x: 0, y: 0 }, { x: 200, y: 0 }, { x: 200, y: 150 }, { x: 0, y: 150 } ],
    inner: [ [ { x: 50, y: 50 }, { x: 150, y: 50 }, { x: 150, y: 100 }, { x: 50, y: 100 } ] ]
  };
  const polyB = {
    outer: [ { x: 0, y: 0 }, { x: 20, y: 0 }, { x: 20, y: 20 }, { x: 0, y: 20 } ]
  };

  try {
    const result = noFitPolygon(polyAWithHole, polyB, false, false); // No NFPs requested
                        // includeOuterNfp = false, includeHolesNfp = false
    t.deepEqual(result, [], 'Should return an empty array when no NFPs are requested');
    t.pass();
  } catch (e) {
    t.fail(`NFP calculation failed: ${e.message}`);
  }
});

test('Input polygon with empty outer path should throw error', (t) => {
  const polyA = { outer: [] }; // Invalid: empty outer path
  const polyB = { outer: [ { x: 0, y: 0 }, { x: 1, y: 0 }, { x: 1, y: 1 }, { x: 0, y: 1 } ] };

  try {
    noFitPolygon(polyA, polyB, true, false);
    t.fail('Expected an error for empty outer path in polygonA, but noFitPolygon succeeded.');
  } catch (e) {
    t.true(e.message.includes('Outer polygon path cannot be empty'), 'Error message should indicate outer path emptiness.');
    console.log("Error for empty outer path (polyA):", e.message);
    t.pass();
  }

  const polyAValid = { outer: [ { x: 0, y: 0 }, { x: 10, y: 0 }, { x: 10, y: 10 }, { x: 0, y: 10 } ] };
  const polyBEmptyOuter = { outer: [] }; // Invalid: empty outer path

   try {
    noFitPolygon(polyAValid, polyBEmptyOuter, true, false);
    t.fail('Expected an error for empty outer path in polygonB, but noFitPolygon succeeded.');
  } catch (e) {
    // The error for polygonB's outer path being empty might be different as it's handled
    // during the creation of `multi_poly_b` inside the Rust code, not during `try_into`.
    // The current Rust code for polygonB's conversion to `multi_poly_b` doesn't explicitly check
    // for empty outer path before creating BabushkaKernelPolygon, which might lead to a panic
    // or different error from Babushka. For now, let's check if an error is thrown.
    t.truthy(e, "An error should be thrown for empty outer path in polygonB");
    console.log("Error for empty outer path (polyB):", e.message);
    // A more specific check could be added if the Rust error is consistent.
    // e.g. t.true(e.message.includes('specific error from babushka or napi_path_to_babushka_polygon'));
    t.pass();
  }
});

test('Input polygon with an empty inner hole path should throw error', (t) => {
  const polyAWithEmptyHole = {
    outer: [ { x: 0, y: 0 }, { x: 200, y: 0 }, { x: 200, y: 150 }, { x: 0, y: 150 } ],
    inner: [ [] ] // Invalid: a hole path is empty
  };
  const polyB = { outer: [ { x: 0, y: 0 }, { x: 20, y: 0 }, { x: 20, y: 20 }, { x: 0, y: 20 } ] };

  try {
    noFitPolygon(polyAWithEmptyHole, polyB, true, true);
    t.fail('Expected an error for empty inner hole path, but noFitPolygon succeeded.');
  } catch (e) {
    t.true(e.message.includes('Inner hole path cannot be empty'), 'Error message should indicate inner hole path emptiness.');
    console.log("Error for empty inner hole path:", e.message);
    t.pass();
  }
});
