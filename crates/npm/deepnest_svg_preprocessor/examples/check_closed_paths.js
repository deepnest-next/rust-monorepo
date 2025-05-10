const { pointsOnSvgPathWithClosedInfo } = require('../');

// Test SVG paths
const paths = [
  {
    name: "Explicitly closed square (Z command)",
    path: "M10,10 L20,10 L20,20 L10,20 Z" 
  },
  {
    name: "Implicitly closed square (last point equals first)",
    path: "M10,10 L20,10 L20,20 L10,20 L10,10"
  },
  {
    name: "Open square",
    path: "M10,10 L20,10 L20,20 L10,20"
  },
  {
    name: "Circle (always closed)",
    path: "M50,25 A25,25 0 1,1 50,24.9 Z"
  },
  {
    name: "Complex gear with Z command",
    path: "M100,100 L120,100 L130,110 L120,120 L100,120 Z M150,150 L170,150 L180,160 L170,170 L150,170 Z"
  }
];

// Test each path and check closed status
paths.forEach(({ name, path }) => {
  console.log(`\nProcessing: ${name}`);
  try {
    const result = pointsOnSvgPathWithClosedInfo(path);
    console.log(`Detected ${result.points.length} subpaths`);
    
    result.points.forEach((points, index) => {
      console.log(`  Subpath ${index + 1}: ${points.length} points, closed: ${result.closed[index]}`);
      
      // Print first and last point to visually verify
      if (points.length >= 2) {
        const first = points[0];
        const last = points[points.length - 1];
        console.log(`    First point: (${first.x.toFixed(2)}, ${first.y.toFixed(2)})`);
        console.log(`    Last point:  (${last.x.toFixed(2)}, ${last.y.toFixed(2)})`);
        
        // Calculate distance between first and last point
        const distance = Math.sqrt(
          Math.pow(last.x - first.x, 2) + 
          Math.pow(last.y - first.y, 2)
        );
        console.log(`    Distance between first and last point: ${distance.toFixed(6)}`);
      }
    });
  } catch (err) {
    console.error(`  Error processing path: ${err.message}`);
  }
});

console.log("\nAll paths processed.");
