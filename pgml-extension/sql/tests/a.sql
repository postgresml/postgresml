
CREATE OR REPLACE FUNCTION py_get_gd()
RETURNS TEXT
AS $$
    return GD.get("pgml.setting", None);
$$ LANGUAGE plpython3u;

CREATE OR REPLACE FUNCTION py_get_sd()
RETURNS TEXT
AS $$
    return SD.get("pgml.setting", None);
$$ LANGUAGE plpython3u;

CREATE OR REPLACE FUNCTION py_set_gd(value TEXT)
RETURNS VOID
AS $$
    GD["pgml.setting"] = value
$$ LANGUAGE plpython3u;

CREATE OR REPLACE FUNCTION py_set_sd(value TEXT)
RETURNS VOID
AS $$
    SD["pgml.setting"] = value
$$ LANGUAGE plpython3u;


select py_get_gd();
select py_get_sd();

select py_set_gd('new_gd_value');
select py_set_sd('new_sd_value');

select py_get_gd();
select py_get_sd();
