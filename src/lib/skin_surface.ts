
type RGB = readonly [number, number, number];

const distance = (a: RGB, b: RGB): number => Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]);
const isSkinTone = ([red, green, blue]: RGB): boolean =>
  red >= 60 && green >= 28 && red >= green * .9 && red >= blue * 1.08
  && red - blue >= 12 && Math.max(red, green, blue) - Math.min(red, green, blue) < 160;
const isEyeWhite = ([red, green, blue]: RGB): boolean =>
  Math.min(red, green, blue) >= 130 && Math.max(red, green, blue) - Math.min(red, green, blue) <= 75;



export function prepareSkinSurface(canvas: HTMLCanvasElement, fallback: boolean, whiteEyes = false): {open: ImageData; closed: ImageData | null} {
  const context = canvas.getContext('2d', {willReadFrequently: true});
  if (!context) throw new Error('No se pudo preparar la superficie de la skin.');
  const width = canvas.width, height = canvas.height;
  const open = context.getImageData(0, 0, width, height);

  if (width < 64 || width % 64 !== 0 || height !== width) return {open, closed: null};
  const scale = width / 64;
  const inner = (x: number, y: number): boolean =>
    (y < 16 && x < 32)
    || (y >= 16 && y < 32)
    || (y >= 48 && y < 64 && x >= 16 && x < 48);

  for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
    const index = (y * width + x) * 4 + 3;
    const alpha = open.data[index];
    if (inner(x / scale, y / scale)) open.data[index] = 255;


    else if (fallback) open.data[index] = alpha === 255 ? 255 : 0;

    else open.data[index] = alpha >= 128 ? 255 : 0;
  }
  if (whiteEyes) {


    for (const x of [9, 10, 13, 14]) {
      for (let dy = 0; dy < scale; dy++) for (let dx = 0; dx < scale; dx++) {
        const base = (((12 * scale + dy) * width) + x * scale + dx) * 4;
        const hat = (((12 * scale + dy) * width) + (x + 32) * scale + dx) * 4;
        open.data.set([255, 255, 255, 255], base);
        open.data[hat + 3] = 0;
      }
    }
    context.putImageData(open, 0, 0);
    return {open, closed: null};
  }
  context.putImageData(open, 0, 0);

  const cell = (x: number, y: number): RGB => {
    let red = 0, green = 0, blue = 0;
    for (let dy = 0; dy < scale; dy++) for (let dx = 0; dx < scale; dx++) {
      const index = (((y * scale + dy) * width) + x * scale + dx) * 4;
      red += open.data[index]; green += open.data[index + 1]; blue += open.data[index + 2];
    }
    const count = scale * scale;
    return [red / count, green / count, blue / count];
  };
  const covered = (x: number, y: number): boolean => {

    for (let dy = 0; dy < scale; dy++) for (let dx = 0; dx < scale; dx++) {
      if (open.data[(((y * scale + dy) * width) + (x + 32) * scale + dx) * 4 + 3]) return true;
    }
    return false;
  };
  const eyeColumns = [9, 10, 13, 14];



  if (eyeColumns.some(x => covered(x, 12))) return {open, closed: null};
  const whiteLeft = cell(9, 12), pupilLeft = cell(10, 12);
  const pupilRight = cell(13, 12), whiteRight = cell(14, 12);
  if (!isEyeWhite(whiteLeft) || !isEyeWhite(whiteRight)
    || distance(whiteLeft, whiteRight) > 80
    || distance(whiteLeft, pupilLeft) < 42 || distance(whiteRight, pupilRight) < 42
    || distance(pupilLeft, pupilRight) > 100) return {open, closed: null};

  const lidRows = new Map<number, number>();
  for (const x of eyeColumns) {
    const above = cell(x, 11), below = cell(x, 13), eye = cell(x, 12);

    const sourceY = isSkinTone(below) && distance(eye, below) >= 32 ? 13
      : isSkinTone(above) && distance(eye, above) >= 32 ? 11 : undefined;
    if (sourceY === undefined) return {open, closed: null};
    lidRows.set(x, sourceY);
  }
  const closed = new ImageData(new Uint8ClampedArray(open.data), width, height);
  for (const x of eyeColumns) {
    const sourceY = lidRows.get(x)!;
    for (let dy = 0; dy < scale; dy++) for (let dx = 0; dx < scale; dx++) {
      const source = (((sourceY * scale + dy) * width) + x * scale + dx) * 4;
      const target = (((12 * scale + dy) * width) + x * scale + dx) * 4;

      closed.data.set(open.data.subarray(source, source + 4), target);
    }
  }
  return {open, closed};
}
