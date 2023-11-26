"""Load data into Redis so we can use it as a feature store."""
import redis
import pandas as pd
import json

def upload():
	r = redis.Redis(host="localhost", port=6379, db=0)

	df = pd.read_csv("~/Desktop/flights_sub.csv")

	print("Uploading data to feature store")

	pipeline = r.pipeline()
	i = 0

	# Upload data to feature store
	for row in df.iterrows():
		key = json.dumps(row[1][["originairportid", "year", "month", "dayofmonth"]].tolist())
		value = json.dumps(row[1][[
		"year",
		"quarter",
		"month",
		"distance",
		"dayofweek",
		"dayofmonth",
		"flight_number_operating_airline",
		"originairportid",
		"destairportid",
		"tail_number"]].tolist())

		pipeline.lpush(key, value)

		if i % 10000 == 0:
			pipeline.execute()
			print("Loaded 10k rows")

		i += 1

	try:
		pipeline.execute()
	except Exception:
		pass


if __name__ == "__main__":
	upload()
