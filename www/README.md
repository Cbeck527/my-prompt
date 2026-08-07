# my-prompt website

This directory contains the static Astro site for
`https://cmb.software/my-prompt/`. It deploys as a Cloudflare Workers Static
Assets project with no Worker script and no browser-side JavaScript.

## Develop

The repository's Nix development shell provides Node.js 24 and pnpm. The
`packageManager` field in `package.json` selects the pinned pnpm release.

```console
nix develop
cd www
pnpm install --frozen-lockfile
pnpm dev
```

Astro serves the site at `http://localhost:4321/my-prompt/`.

## Check

```console
pnpm check
pnpm build
pnpm exec wrangler deploy --dry-run
```

`pnpm preview` builds the site and starts Wrangler's local server. Check
`/my-prompt/` for the site and a missing path such as `/my-prompt/missing/` for
a 404 response.

## Hosting contract

This project owns only the Cloudflare route `cmb.software/my-prompt/*`. The
separate index Worker must:

- own `cmb.software` as a Cloudflare Custom Domain; and
- redirect exact `/my-prompt` to `/my-prompt/` with a query-preserving `308`.

Deploy the index Worker first so the zone has proxied DNS for the route. This
project does not provide a `workers.dev` hostname and does not handle the bare
domain or exact `/my-prompt` path.

## Deploy

Authenticate Wrangler for the Cloudflare account that owns `cmb.software`, then
deploy manually from this directory:

```console
pnpm deploy
```

That command builds the nested `dist/my-prompt/` output before publishing it.
