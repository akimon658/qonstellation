import { define } from "../lib/define.ts"

export default define.page(({ Component }) => (
  <html lang="ja">
    <head>
      <meta charSet="UTF-8" />
      <meta name="viewport" content="width=device-width, initial-scale=1.0" />
      <title>Qonstellation</title>
    </head>

    <body>
      <Component />
    </body>
  </html>
))
