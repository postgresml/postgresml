from collections import deque

execute_results = deque()


def quote_literal(literal):
    return "'" + literal + "'"


def execute(sql, lines=0):
    if len(execute_results) > 0:
        result = execute_results.popleft()
        return result
    else:
        return []


def add_mock_result(result):
    execute_results.append(result)
