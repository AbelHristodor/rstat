from flask import Flask, jsonify

app = Flask(__name__)

@app.route('/health', methods=['GET'])
def health():
    return jsonify({"status": "healthy"}), 200

@app.route('/error', methods=['GET'])
def error():
    return jsonify({"error": "Bad Gateway"}), 502

if __name__ == '__main__':
    app.run(debug=True)