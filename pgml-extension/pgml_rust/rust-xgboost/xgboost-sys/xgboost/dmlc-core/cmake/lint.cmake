get_filename_component(CMAKE_SOURCE_DIR "${CMAKE_CURRENT_LIST_DIR}/.." ABSOLUTE)
if(NOT MSVC)
    set(LINT_COMMAND ${CMAKE_SOURCE_DIR}/scripts/lint.py)
else()
    if((NOT PYTHON_EXECUTABLE))
         message(FATAL_ERROR "Cannot lint without python")
    endif()
    # format output so VS can bring us to the offending file/line
	set(LINT_COMMAND ${PYTHON_EXECUTABLE} ${CMAKE_SOURCE_DIR}/scripts/lint.py)
endif()

cmake_policy(SET CMP0009 NEW)  # suppress cmake warning
string(REPLACE " " ";" LINT_DIRS ${LINT_DIRS})
string(REPLACE " " ";" EXCLUDE_PATH ${EXCLUDE_PATH})
execute_process(
    COMMAND ${LINT_COMMAND} ${PROJECT_NAME} all ${LINT_DIRS} --exclude_path=${EXCLUDE_PATH}
	WORKING_DIRECTORY ${PROJECT_SOURCE_DIR}
    ERROR_VARIABLE LINT_OUTPUT
    ERROR_STRIP_TRAILING_WHITESPACE
)
message(STATUS ${LINT_OUTPUT})