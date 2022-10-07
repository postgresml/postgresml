#!/usr/bin/env python3
#
# Test serializing the XGBoost model in Python
# and using it in Rust/PostgresML.
#
import xgboost as xgb
import pandas as pd

if __name__ == "__main__":
	data = pd.read_csv("diabetes.csv")

	# age,sex,bmi,bp,s1,s2,s3,s4,s5,s6,target
	dtrain = xgb.DMatrix(data)

	linear = xgb.XGBRegressor(objective='reg:linear',
	    n_estimators=1000,
	    learning_rate=0.10,
	    subsample=0.5,
	    colsample_bytree=1, 
	    max_depth=5,
	)

	X, y = data.loc[:,['age', 'sex']], data.loc[:,['target']]	
	bst = linear.fit(X, y)
	bst.save_model("/tmp/xgboost_model_python.bin")

