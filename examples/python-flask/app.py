from flask import Flask, Response, jsonify, request

app = Flask(__name__)


@app.get("/api/health")
def health():
    return jsonify(
        ok=True,
        service="flask",
        host=request.headers.get("Host", ""),
        forwarded_for=request.headers.get("X-Forwarded-For", ""),
    )


@app.post("/api/echo")
def echo():
    return jsonify(received=request.get_json(silent=True) or {})


@app.get("/api/stream")
def stream():
    def chunks():
        yield "gate\n"
        yield "flask\n"

    return Response(chunks(), mimetype="text/plain")


if __name__ == "__main__":
    app.run(host="127.0.0.1", port=5000)
