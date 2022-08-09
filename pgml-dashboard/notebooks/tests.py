from django.test import TestCase
from notebooks.models import *


class TestNodebookLine(TestCase):
    def setUp(self):
        self.notebook = Notebook(name="Test")

    def test_markdown(self):
        markdown = """
# Title
Paragraph goes here.

## Subtitle

```
SELECT * FROM test;
```

1. One
2. Two
3. Three

- List
- Of
- Items

### Smaller title
Hello world!
        """

        line = NotebookLine(notebook=self.notebook, line_type=NotebookLine.MARKDOWN, contents=markdown)
        html = line.html()

        self.assertIn("<p>", html)
        self.assertIn("<code>", html)
        self.assertIn("<li>\n<p>List", html)

    def test_plain_text(self):
        plain_text = """
        Hey there friends!
        """

        line = NotebookLine(notebook=self.notebook, line_type=NotebookLine.PLAIN_TEXT, contents=plain_text)
        html = line.html()

        self.assertIn("Hey there friends!", html)
        self.assertNotIn("<p>", html)

    def test_sql(self):
        sql = """
        SELECT 1 AS one, 2 as two, 3 as three, 'text' AS _text
        """

        line = NotebookLine(notebook=self.notebook, line_type=NotebookLine.SQL, contents=sql)
        html = line.html()

        self.assertIn("<table>", html)
        self.assertIn("<td>1</td>", html)
        self.assertIn("<td>one</td>", html)
