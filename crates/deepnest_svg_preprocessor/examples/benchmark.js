const { pointsOnSvgPath } = require('..');

// Test paths with varying complexity
const PATHS = [
  {
    name: "Simple Line",
    path: "M10,20 L30,40"
  },
  {
    name: "Bezier Curve",
    path: "M10,20 C30,40,50,60,70,80"
  },
  {
    name: "Arc Path",
    path: "M10,20 A30,40 0 1,1 70,80"
  },
  {
    name: "Complex Gear",
    path: "M 2034.3 265.879 l 9.293 45.242 c 17.984 2.7520000000000002 35.611 7.475 52.562 14.084 l 30.669 -34.534 c 17.162 7.803 33.526 17.251 48.865 28.212 l -14.573 43.827 c 14.199 11.375 27.103 24.279 38.479 38.478 l 43.826 -14.572 c 10.961 15.338000000000001 20.409 31.703 28.212 48.864 l -34.533 30.669 c 6.609 16.951 11.332 34.578 14.084 52.563 l 45.241 9.293 c 1.823 18.764 1.823 37.66 0 56.424 l -45.241 9.293 c -2.7520000000000002 17.984 -7.475 35.611 -14.084 52.563 l 34.533 30.668 c -7.803 17.162 -17.251 33.527 -28.212 48.865 l -43.826 -14.573 c -11.376 14.199 -24.28 27.103 -38.479 38.479 l 14.573 43.826 c -15.339 10.961 -31.703 20.409 -48.865 28.212 l -30.669 -34.533 c -16.951 6.609 -34.578 11.332 -52.562 14.084 l -9.293 45.241 c -18.764 1.823 -37.661 1.823 -56.424 0 l -9.293 -45.241 c -17.985 -2.7520000000000002 -35.612 -7.475 -52.563 -14.084 l -30.669 34.533 c -17.161 -7.803 -33.526 -17.251 -48.864 -28.212 l 14.572 -43.826 c -14.199 -11.376 -27.103 -24.28 -38.478 -38.479 l -43.827 14.573 c -10.961 -15.338000000000001 -20.409 -31.703 -28.212 -48.865 l 34.534 -30.668 c -6.609 -16.952 -11.333 -34.579 -14.085 -52.563 l -45.241 -9.293 c -1.823 -18.764 -1.823 -37.66 0 -56.424 l 45.241 -9.293 c 2.7520000000000002 -17.985 7.476 -35.612 14.085 -52.563 l -34.534 -30.669 c 7.803 -17.161 17.251 -33.526 28.212 -48.864 l 43.827 14.572 c 11.375 -14.199 24.279 -27.103 38.478 -38.478 l -14.572 -43.827 c 15.338000000000001 -10.961 31.703 -20.409 48.864 -28.212 l 30.669 34.534 c 16.951 -6.609 34.578 -11.332 52.563 -14.084 l 9.293 -45.242 c 18.763 -1.823 37.66 -1.823 56.424 0 z"
  },
  {
    name: "Logo Path",
    path: "M532.094,806.71c6.595,91.184 45.177,175.149 106.414,244.044c-18.85,3.17 -38.222,4.822 -57.982,4.822c-189.194,-0 -342.795,-151.386 -342.795,-337.85c0,-186.465 153.601,-337.851 342.795,-337.851c57.886,0 112.441,14.172 160.283,39.185c-75.737,49.42 -135.471,115.393 -171.506,191.444c-11.119,-4.197 -23.167,-6.494 -35.747,-6.494c-55.939,0 -101.355,45.416 -101.355,101.355c0,55.452 44.627,100.562 99.893,101.345Z"
  }
];

// Configuration for different benchmarks
const CONFIGS = [
  { name: "Default Tolerance", tolerance: undefined, simplify: undefined },
  { name: "Higher Tolerance (0.5)", tolerance: 0.5, simplify: undefined },
  { name: "With Simplification", tolerance: 0.5, simplify: 1.0 }
];

// Helper function to run a benchmark
function runBenchmark(pathObj, config, iterations = 20) {
  const { name: pathName, path } = pathObj;
  const { name: configName, tolerance, simplify } = config;
  
  console.log(`\nRunning benchmark: ${pathName} - ${configName}`);
  console.log(`Parameters: tolerance=${tolerance}, simplify=${simplify}`);
  
  // Warm-up run
  pointsOnSvgPath(path, tolerance, simplify);
  
  const times = [];
  let totalPoints = 0;
  
  for (let i = 0; i < iterations; i++) {
    const start = process.hrtime.bigint();
    const pointSets = pointsOnSvgPath(path, tolerance, simplify);
    const end = process.hrtime.bigint();
    
    const pointCount = pointSets.reduce((sum, set) => sum + set.length, 0);
    totalPoints += pointCount;
    
    times.push(Number(end - start) / 1_000_000); // Convert to milliseconds
  }
  
  // Calculate statistics
  times.sort((a, b) => a - b);
  const mean = times.reduce((sum, time) => sum + time, 0) / times.length;
  const median = times.length % 2 === 0
    ? (times[times.length / 2 - 1] + times[times.length / 2]) / 2
    : times[Math.floor(times.length / 2)];
  const min = times[0];
  const max = times[times.length - 1];
  const avgPointCount = totalPoints / iterations;
  
  console.log(`Results after ${iterations} iterations:`);
  console.log(`  Average time: ${mean.toFixed(3)} ms`);
  console.log(`  Median time: ${median.toFixed(3)} ms`);
  console.log(`  Min time: ${min.toFixed(3)} ms`);
  console.log(`  Max time: ${max.toFixed(3)} ms`);
  console.log(`  Average point count: ${avgPointCount.toFixed(0)}`);
  console.log(`  Points per millisecond: ${(avgPointCount / mean).toFixed(2)}`);
  
  return { mean, median, min, max, avgPointCount };
}

// Main benchmark runner
async function runAllBenchmarks() {
  console.log("SVG Path Processing Benchmark");
  console.log("============================");
  
  const results = [];
  
  for (const path of PATHS) {
    for (const config of CONFIGS) {
      const result = runBenchmark(path, config);
      results.push({
        path: path.name,
        config: config.name,
        ...result
      });
    }
  }
  
  // Memory usage test - run the complex path repeatedly
  console.log("\nRunning memory usage test (repeated processing)...");
  const complexPath = PATHS[3].path;
  const iterations = 100;
  
  // Start monitoring memory
  const memBefore = process.memoryUsage();
  console.log("Memory before:", formatMemory(memBefore));
  
  // Run the test
  for (let i = 0; i < iterations; i++) {
    pointsOnSvgPath(complexPath, 0.5, undefined);
  }
  
  // Check memory after
  const memAfter = process.memoryUsage();
  console.log("Memory after:", formatMemory(memAfter));
  console.log("Difference:", formatMemoryDiff(memBefore, memAfter));
  
  // Summary 
  console.log("\nBenchmark Summary");
  console.log("================");
  console.table(results.map(r => ({
    Path: r.path,
    Config: r.config,
    "Mean (ms)": r.mean.toFixed(2),
    "Median (ms)": r.median.toFixed(2),
    "Points": r.avgPointCount.toFixed(0),
    "Points/ms": (r.avgPointCount / r.mean).toFixed(2)
  })));
}

// Helper to format memory usage numbers
function formatMemory(mem) {
  return {
    rss: `${(mem.rss / 1024 / 1024).toFixed(2)} MB`,
    heapTotal: `${(mem.heapTotal / 1024 / 1024).toFixed(2)} MB`,
    heapUsed: `${(mem.heapUsed / 1024 / 1024).toFixed(2)} MB`,
    external: `${(mem.external / 1024 / 1024).toFixed(2)} MB`
  };
}

function formatMemoryDiff(before, after) {
  return {
    rss: `${((after.rss - before.rss) / 1024 / 1024).toFixed(2)} MB`,
    heapTotal: `${((after.heapTotal - before.heapTotal) / 1024 / 1024).toFixed(2)} MB`,
    heapUsed: `${((after.heapUsed - before.heapUsed) / 1024 / 1024).toFixed(2)} MB`,
    external: `${((after.external - before.external) / 1024 / 1024).toFixed(2)} MB`
  };
}

// Run the benchmarks
runAllBenchmarks().catch(console.error);
