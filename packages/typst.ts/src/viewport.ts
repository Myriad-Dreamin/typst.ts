/** @internal */
export type PageViewportParameters = {
  /**
   * - The xMin, yMin, xMax and
   * yMax coordinates.
   */
  viewBox: Array<number>;
  /**
   * - The scale of the viewport.
   */
  scale: number;
  /**
   * - The rotation, in degrees, of the viewport.
   */
  rotation: number;
  /**
   * - The horizontal, i.e. x-axis, offset. The
   * default value is `0`.
   */
  offsetX?: number | undefined;
  /**
   * - The vertical, i.e. y-axis, offset. The
   * default value is `0`.
   */
  offsetY?: number | undefined;
  /**
   * - If true, the y-axis will not be flipped.
   * The default value is `false`.
   */
  dontFlip?: boolean | undefined;
};

/** @internal */
export class PageViewport {
  viewBox: number[];
  scale: number;
  rotation: number;
  offsetX: number;
  offsetY: number;
  transform: number[];
  width: number;
  height: number;

  constructor({
    viewBox,
    scale,
    rotation,
    offsetX = 0,
    offsetY = 0,
    dontFlip = false,
  }: PageViewportParameters) {
    this.viewBox = viewBox;
    this.scale = scale;
    this.rotation = rotation;
    this.offsetX = offsetX;
    this.offsetY = offsetY;
    const centerX = (viewBox[2] + viewBox[0]) / 2;
    const centerY = (viewBox[3] + viewBox[1]) / 2;
    let rotateA, rotateB, rotateC, rotateD;
    rotation %= 360;
    if (rotation < 0) {
      rotation += 360;
    }
    switch (rotation) {
      case 180:
        rotateA = -1;
        rotateB = 0;
        rotateC = 0;
        rotateD = 1;
        break;
      case 90:
        rotateA = 0;
        rotateB = 1;
        rotateC = 1;
        rotateD = 0;
        break;
      case 270:
        rotateA = 0;
        rotateB = -1;
        rotateC = -1;
        rotateD = 0;
        break;
      case 0:
        rotateA = 1;
        rotateB = 0;
        rotateC = 0;
        rotateD = -1;
        break;
      default:
        throw new Error('PageViewport: Invalid rotation, must be a multiple of 90 degrees.');
    }
    if (dontFlip) {
      rotateC = -rotateC;
      rotateD = -rotateD;
    }
    let offsetCanvasX, offsetCanvasY;
    let width, height;
    if (rotateA === 0) {
      offsetCanvasX = Math.abs(centerY - viewBox[1]) * scale + offsetX;
      offsetCanvasY = Math.abs(centerX - viewBox[0]) * scale + offsetY;
      width = (viewBox[3] - viewBox[1]) * scale;
      height = (viewBox[2] - viewBox[0]) * scale;
    } else {
      offsetCanvasX = Math.abs(centerX - viewBox[0]) * scale + offsetX;
      offsetCanvasY = Math.abs(centerY - viewBox[1]) * scale + offsetY;
      width = (viewBox[2] - viewBox[0]) * scale;
      height = (viewBox[3] - viewBox[1]) * scale;
    }
    this.transform = [
      rotateA * scale,
      rotateB * scale,
      rotateC * scale,
      rotateD * scale,
      offsetCanvasX - rotateA * scale * centerX - rotateC * scale * centerY,
      offsetCanvasY - rotateB * scale * centerX - rotateD * scale * centerY,
    ];
    this.width = width;
    this.height = height;
  }
  get rawDims() {
    const { viewBox } = this;
    return {
      // todo: shadow
      pageWidth: viewBox[2] - viewBox[0],
      pageHeight: viewBox[3] - viewBox[1],
      pageX: viewBox[0],
      pageY: viewBox[1],
    };
  }
  clone({
    scale = this.scale,
    rotation = this.rotation,
    offsetX = this.offsetX,
    offsetY = this.offsetY,
    dontFlip = false,
  } = {}) {
    return new PageViewport({
      viewBox: this.viewBox.slice(),
      scale,
      rotation,
      offsetX,
      offsetY,
      dontFlip,
    });
  }
  static applyTransform(p: number[], m: number[]) {
    const xt = p[0] * m[0] + p[1] * m[2] + m[4];
    const yt = p[0] * m[1] + p[1] * m[3] + m[5];
    return [xt, yt];
  }
  static applyInverseTransform(p: number[], m: number[]) {
    const d = m[0] * m[3] - m[1] * m[2];
    const xt = (p[0] * m[3] - p[1] * m[2] + m[2] * m[5] - m[4] * m[3]) / d;
    const yt = (-p[0] * m[1] + p[1] * m[0] + m[4] * m[1] - m[5] * m[0]) / d;
    return [xt, yt];
  }
  convertToViewportPoint(x: number, y: number) {
    return PageViewport.applyTransform([x, y], this.transform);
  }
  convertToViewportRectangle(rect: number[]) {
    const topLeft = PageViewport.applyTransform([rect[0], rect[1]], this.transform);
    const bottomRight = PageViewport.applyTransform([rect[2], rect[3]], this.transform);
    return [topLeft[0], topLeft[1], bottomRight[0], bottomRight[1]];
  }
  convertToPdfPoint(x: number, y: number) {
    return PageViewport.applyInverseTransform([x, y], this.transform);
  }
}
