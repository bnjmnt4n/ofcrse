import type { APIRoute } from "astro";
import { renderPng } from "../../../utils/renderPng";

export const GET: APIRoute = async () => {
  const image = await renderPng({
    type: "div",
    props: {
      children: [
        {
          type: "div",
          props: {
            children: "ofcrse",
            style: {
              whiteSpace: "pre",
              fontSize: 200,
              fontWeight: 400,
              lineHeight: 1,
              letterSpacing: -2,
              marginBottom: 60,
              borderBottom: "8px solid #128886",
            },
          },
        },
        {
          type: "div",
          props: {
            children: "By Benjamin Tan",
            style: {
              color: "#83758c",
              fontSize: 70,
              fontWeight: 400,
            },
          },
        },
      ],
      style: {
        display: "flex",
        height: "100%",
        width: "100%",
        padding: "100px 80px",
        flexDirection: "column",
        alignItems: "flex-start",
        justifyContent: "flex-end",
        backgroundColor: "#fdfdfa",
        color: "#301940",
      },
    },
  });

  return new Response(image, {
    status: 200,
    headers: {
      "Content-Type": "image/png",
    },
  });
};
