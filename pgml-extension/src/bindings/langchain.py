from langchain.text_splitter import (
    CharacterTextSplitter,
    LatexTextSplitter,
    MarkdownTextSplitter,
    NLTKTextSplitter,
    PythonCodeTextSplitter,
    RecursiveCharacterTextSplitter,
    SpacyTextSplitter,
)
import json

SPLITTERS = {
    "character": CharacterTextSplitter,
    "latex": LatexTextSplitter,
    "markdown": MarkdownTextSplitter,
    "nltk": NLTKTextSplitter,
    "python": PythonCodeTextSplitter,
    "recursive_character": RecursiveCharacterTextSplitter,
    "spacy": SpacyTextSplitter,
}


def chunk(splitter, text, args):
    kwargs = json.loads(args)

    if splitter in SPLITTERS:
        return SPLITTERS[splitter](**kwargs).split_text(text)
    else:
        raise ValueError("Unknown splitter: {}".format(splitter))
