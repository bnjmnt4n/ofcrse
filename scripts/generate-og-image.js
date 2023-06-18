const child_process = require("child_process");
const fs = require("fs/promises");
const { default: satori } = require("satori");

(async () => {
  if (process.argv.length < 6) {
    console.log("USAGE: node generate-og-image.js FONT_TTF TITLE SUBTITLE OUTPUT [TITLE_FONT_SIZE] [SUBTITLE_FONT_SIZE]");
    return;
  }

  const [, , FONT_TTF, TITLE, SUBTITLE, OUTPUT_FILE, TITLE_FONT_SIZE = 100, SUBTITLE_FONT_SIZE = 50] = process.argv;

  const font = await fs.readFile(FONT_TTF);

  const svg = await satori(
    {
      type: "div",
      props: {
        children: [
          {
            type: "div",
            props: {
              children: TITLE,
              style: {
                whiteSpace: 'pre',
                fontSize: Number(TITLE_FONT_SIZE),
                fontWeight: 400,
                lineHeight: 1,
                letterSpacing: -2,
                marginBottom: 60,
              }
            },
          },
          {
            type: "div",
            props: {
              children: SUBTITLE,
              style: {
                color: "#83758c",
                fontSize: Number(SUBTITLE_FONT_SIZE),
                fontWeight: 400,
              }
            },
          }
        ],
        style: {
          display: "flex",
          height: "100%",
          width: "100%",
          padding: "120px 80px",
          flexDirection: "column",
          alignItems: "flex-start",
          justifyContent: "flex-end",
          backgroundColor: "#fdfdfa",
          color: "#301940",
        },
      },
    },
    {
      width: 1200,
      height: 630,
      fonts: [
        {
          name: "Font",
          data: font,
          weight: 400,
          style: "normal",
        },
      ],
    },
  );

  // await fs.writeFile(OUTPUT, svg);
  child_process.spawnSync("resvg", ["-", OUTPUT_FILE], { input: svg });
})();
