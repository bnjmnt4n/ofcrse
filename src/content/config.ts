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
      .optional()
  }),
});

export const collections = { articles };
