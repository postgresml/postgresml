#!/bin/bash
# Copyright (c) 2020, NVIDIA CORPORATION.
#####################
# GPUTreeShap Style Tester #
#####################

# Ignore errors and set path
set +e
PATH=/opt/conda/bin:$PATH
RETVAL="0"

# Activate common conda env
. /opt/conda/etc/profile.d/conda.sh
conda activate rapids

# Check for a consistent code format
pip install cpplint
FORMAT=`cpplint --recursive GPUTreeShap tests example benchmark 2>&1`
FORMAT_RETVAL=$?
if [ "$RETVAL" = "0" ]; then
  RETVAL=$FORMAT_RETVAL
fi

# Output results if failure otherwise show pass
if [ "$FORMAT_RETVAL" != "0" ]; then
  echo -e "\n\n>>>> FAILED: cpplint format check; begin output\n\n"
  echo -e "$FORMAT"
  echo -e "\n\n>>>> FAILED: cpplint format check; end output\n\n"
else
  echo -e "\n\n>>>> PASSED: cpplint format check\n\n"
fi

exit $RETVAL
