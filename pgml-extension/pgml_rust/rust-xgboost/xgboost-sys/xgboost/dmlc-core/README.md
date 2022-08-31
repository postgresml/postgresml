Distributed Machine Learning Common Codebase
============================================

[![Build Status](https://github.com/dmlc/dmlc-core/workflows/continuous%20build/badge.svg)](https://github.com/dmlc/dmlc-core/actions)
[![Documentation Status](https://readthedocs.org/projects/dmlc-core/badge/?version=latest)](http://dmlc-core.readthedocs.org/en/latest/)
[![GitHub license](http://dmlc.github.io/img/apache2.svg)](./LICENSE)


DMLC-Core is the backbone library to support all DMLC projects, offers the bricks to build efficient and scalable distributed machine learning libraries.

Developer Channel [![Join the chat at https://gitter.im/dmlc/dmlc-core](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/dmlc/dmlc-core?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)


What's New
----------
* [Note on Parameter Module for Machine Learning](http://dmlc-core.readthedocs.org/en/latest/parameter.html)


Contents
--------
* [Documentation and Tutorials](http://dmlc-core.readthedocs.org/en/latest/)
* [Contributing](#contributing)

Known Issues
------------
* RecordIO format is not portable across different processor endians. So it is not possible to save RecordIO file on a x86 machine and then load it on a SPARC machine, because x86 is little endian while SPARC is big endian.


Contributing
------------

Contributing to dmlc-core is welcomed! dmlc-core follows google's C style guide. If you are interested in contributing, take a look at [feature wishlist](https://github.com/dmlc/dmlc-core/labels/feature%20wishlist) and open a new issue if you like to add something.

* DMLC-Core uses C++11 standard. Ensure that your C++ compiler supports C++11.
* Try to introduce minimum dependency when possible

### CheckList before submit code
* Type ```make lint``` and fix all the style problems.
* Type ```make doc``` and fix all the warnings.

NOTE
----
deps:

libcurl4-openssl-dev
