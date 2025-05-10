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
