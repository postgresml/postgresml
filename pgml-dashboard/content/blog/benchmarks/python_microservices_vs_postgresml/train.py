import xgboost as xgb
import pandas as pd
from sklearn.model_selection import train_test_split
from sklearn.metrics import r2_score
import redis
import json

def train():
	estimator = xgb.XGBRegressor(n_estimators=25)
	df = pd.read_csv("~/Desktop/flights.csv")

	X, y = df[[
		"year",
		"quarter",
		"month",
		"distance",
		"dayofweek",
		"dayofmonth",
		"flight_number_operating_airline",
		"originairportid",
		"destairportid",
		"tail_number"]], df[["depdelayminutes"]]

	X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.25)

	estimator.fit(X_train, y_train)
	y_hat = estimator.predict(X_test)

	r2 = r2_score(y_test, y_hat)

	estimator.save_model("model.bin")

if __name__ == "__main__":
	train()
