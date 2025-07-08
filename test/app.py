from flask import Flask, jsonify
import random

app = Flask(__name__)

@app.route('/health', methods=['GET'])
def health():
    # status = random.choice([("healthy", 200), ("unhealthy", 500)])
    # return jsonify({"status": status[0]}), status[1]
    return jsonify({"status": "healthy"}), 200

@app.route('/error', methods=['GET'])
def error():
    return jsonify({"error": "Bad Gateway"}), 502

if __name__ == '__main__':
    app.run(debug=True)