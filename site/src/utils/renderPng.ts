import fs from "node:fs";
import path from "node:path";
import satori from "satori";
import sharp from "sharp";

const FONT_PATH = path.join(process.cwd(), "public/assets/fonts/basiersquarenarrow-regular-webfont.ttf");
const font = fs.readFileSync(FONT_PATH);

export async function renderPng(element: any) {
  const svg = await satori(
    element,
    {
      width: 1200,
      height: 630,
      fonts: [
        {
          name: "Basier Square Narrow",
          data: font,
          weight: 400,
          style: "normal",
        },
      ],
    },
  );

  const png = sharp(Buffer.from(svg)).png();
  return await png.toBuffer();
}
