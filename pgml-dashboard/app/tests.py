from django.test import TestCase
from notebooks.models import *


class TestNotebookCell(TestCase):
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

        cell = NotebookCell(notebook=self.notebook, cell_type=NotebookCell.MARKDOWN, contents=markdown)
        html = cell.html()

        self.assertIn("<p>", html)
        self.assertIn("<code>", html)
        self.assertIn("<li>\n<p>List", html)

    def test_plain_text(self):
        plain_text = """
        Hey there friends!
        """

        cell = NotebookCell(notebook=self.notebook, cell_type=NotebookCell.PLAIN_TEXT, contents=plain_text)
        html = cell.html()

        self.assertIn("Hey there friends!", html)
        self.assertNotIn("<p>", html)

    def test_sql(self):
        sql = """
        SELECT 1 AS one, 2 as two, 3 as three, 'text' AS _text
        """

        cell = NotebookCell(notebook=self.notebook, cell_type=NotebookCell.SQL, contents=sql)
        html = cell.html()

        self.assertIn("<table>", html)
        self.assertIn("<td>1</td>", html)
        self.assertIn("<td>one</td>", html)
