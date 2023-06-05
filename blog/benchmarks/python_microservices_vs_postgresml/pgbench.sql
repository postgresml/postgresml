SELECT pgml.predict('r2',
	ARRAY[year, quarter, month, distance, dayofweek, dayofmonth, flight_number_operating_airline, originairportid, destairportid, tail_number]
) FROM flights_delay_mat WHERE originairportid = 14869 and year = 2019 and month = 8 and dayofmonth = 31 LIMIT 1;
