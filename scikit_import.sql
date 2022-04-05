CREATE OR REPLACE FUNCTION scikit_learn_import()
RETURNS TEXT
AS $$
    import sklearn
    return sklearn.__version__
$$ LANGUAGE plpython3u;

