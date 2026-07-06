# Gate Website

World-class product website for Gate, built with Next.js App Router, React, TypeScript, Tailwind CSS, shadcn/ui-style primitives, motion, Lenis and Lucide React.

## Scripts

```bash
npm install
npm run dev
npm run typecheck
npm run lint
npm run build
```

The development server defaults to:

```text
http://127.0.0.1:3000
```

## Structure

```text
app/                    Next.js App Router entry, metadata and global CSS
components/landing/     Homepage sections and interaction logic
components/ui/          shadcn/ui-style primitives
lib/                    Shared utilities
public/                 Static brand assets
```

## Notes

- Dark theme is the default.
- Homepage content uses the repository's actual project name: Gate.
- Performance numbers are presented as engineering targets and benchmark lanes, not released production claims.
- Local build artifacts, dependencies and cache directories are ignored by `website/.gitignore`.
