import test from 'ava'
import { readFileSync } from 'fs'

import { loadSvgString, loadSvgFile } from '../index.js'

function removeWindowsLineEndings(str) {
  return str.replace(/\r\n/g, '\n');
}

test('loadSvgString from native', (t) => {
  const testSvg = readFileSync('./__test__/test.svg', 'utf8');
  const resultSvg = readFileSync('./__test__/result.svg', 'utf8');
  const result = loadSvgString(testSvg);
  t.is(removeWindowsLineEndings(result.result), removeWindowsLineEndings(resultSvg))
})

test('loadSvgFile from native', (t) => {
  const resultSvg = readFileSync('./__test__/result.svg', 'utf8');
  const result = loadSvgFile('./__test__/test.svg');
  t.is(removeWindowsLineEndings(result.result), removeWindowsLineEndings(resultSvg))
})

// usvg stored geometry as f32: a 1000 mm box came back 2834.6457 units wide (deepnest#137).
const pathNumbers = (svg) =>
  [...svg.matchAll(/ d="([^"]*)"/g)].map((m) => m[1].match(/-?[\d.]+(e-?\d+)?/g).map(Number))

const box = (w, h) =>
  `<svg xmlns="http://www.w3.org/2000/svg" width="${w}mm" height="${h}mm" viewBox="0 0 ${w} ${h}"><rect width="${w}" height="${h}"/></svg>`

test('loadSvgString keeps mm sizes exact at 72 units per inch', (t) => {
  for (const [w, h] of [[1000, 1000], [3000, 1500], [123.456, 78.9]]) {
    const result = loadSvgString(box(w, h), 72)
    t.true(result.success)
    const xs = pathNumbers(result.result)[0].filter((_, i) => i % 2 === 0)
    const ys = pathNumbers(result.result)[0].filter((_, i) => i % 2 === 1)
    // The same expression deepnest uses for a sheet typed in mm.
    t.is(Math.max(...xs), w * (72 / 25.4))
    t.is(Math.max(...ys), h * (72 / 25.4))
  }
  t.is(pathNumbers(loadSvgString(box(1000, 1000), 72).result)[0][2], 2834.645669291339)
})

test('loadSvgString applies nested transforms in f64', (t) => {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="10in" height="10in" viewBox="0 0 720 720">
    <g transform="translate(10.123456789 20.987654321)"><g transform="matrix(2 0 0 3 0.5 0.25)">
      <path d="M 0.1 0.2 L 33.333333333 44.444444444"/></g></g></svg>`
  const [p] = pathNumbers(loadSvgString(svg, 72).result)
  const x = (v) => 10.123456789 + (2 * v + 0.5)
  const y = (v) => 20.987654321 + (3 * v + 0.25)
  t.deepEqual(p, [x(0.1), y(0.2), x(33.333333333), y(44.444444444)])
  t.not(Math.fround(p[2]), p[2])
})
