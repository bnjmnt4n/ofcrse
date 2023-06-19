import { defineCollection, z } from "astro:content";

const articles = defineCollection({
  schema: z.object({
    title: z.string(),
    description: z.string(),
    publishedAt: z
      .date()
      .optional(),
    updatedAt: z
      .date()
      .optional(),
    cover: z.boolean()
      .optional()
      .or(z.object({
        title: z.string().optional(),
        subtitle: z.string().optional(),
        titleFontSize: z.number().optional(),
        subtitleFontSize: z.number().optional(),
      }))
      .transform((value) => value === undefined ? true : value),
  }),
});

export const collections = { articles };
