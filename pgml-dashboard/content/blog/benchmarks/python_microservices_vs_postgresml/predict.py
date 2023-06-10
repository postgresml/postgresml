"""A typical minimal Python ML microservice."""
from flask import Flask, jsonify, request
import xgboost as xgb
from redis import Redis
import json
import time

app = Flask(__name__)

model = xgb.XGBRegressor()
model.load_model("model.bin")

redis = Redis(host="localhost", port=6379, db=0)

@app.route("/", methods=["POST"])
def api():
	body = request.json
	key = json.dumps(body)

	# This simulates single prediction.
	# It's possible to batch these up by removing [0]
	# and collecting all elements in the list.
	# Fetching one element is O(1)
	value = redis.lrange(key, 0, 1)[0]
	value = json.loads(value)
	y_hat = model.predict([value])

	return jsonify(y_hat[0].tolist())

if __name__ == "__main__":
	app.run(host="0.0.0.0", port="8000")
