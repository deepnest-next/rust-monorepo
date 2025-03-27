/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface Point {
  x: number
  y: number
}
export interface ConvexHullResult {
  points: Array<Point>
}
export declare function computeConvexHull(points: Array<Point>): ConvexHullResult
export declare function pointsOnSvgPath(path: string, tolerance?: number | undefined | null, distance?: number | undefined | null): Array<Array<Point>>
export declare function pointsOnSvgPathWithClosedInfo(path: string, tolerance?: number | undefined | null, distance?: number | undefined | null): PathResult
export interface LoadSvgResult {
  success: boolean
  result: string
}
export declare function loadSvgString(svgData: string): LoadSvgResult
export declare function loadSvgFile(svgPath: string): LoadSvgResult
/** Information about a processed path including whether it's closed */
export declare class PathResult {
  /** Sets of points that approximate the path */
  points: Array<Array<Point>>
  /** Whether the path is closed (ends with 'Z' command or first point equals last point) */
  closed: Array<boolean>
}
