import express from "express"

const app = express()
const port = Number(process.env.PORT ?? 3000)

app.use(express.json())

app.get("/api/health", (_req, res) => {
  res.json({ ok: true, service: "express", protocol: "http" })
})

app.post("/api/echo", (req, res) => {
  res.json({ received: req.body })
})

app.get("/api/stream", (_req, res) => {
  res.setHeader("Content-Type", "text/plain; charset=utf-8")
  res.setHeader("Transfer-Encoding", "chunked")
  res.write("gate\n")
  setTimeout(() => {
    res.end("express\n")
  }, 100)
})

app.listen(port, "127.0.0.1", () => {
  console.log(`Express example listening on http://127.0.0.1:${port}`)
})
