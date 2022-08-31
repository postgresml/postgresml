# Makfile for easily install dependencies

# List of packages here
.PHONY: gtest

# rules for gtest
/tmp/gtest/include/gtest:
	rm -rf gtest release-1.10.0.zip
	wget https://github.com/google/googletest/archive/release-1.10.0.zip
	unzip release-1.10.0.zip
	mv googletest-release-1.10.0 gtest
	cd gtest; $(CXX) $(CXXFLAGS) -std=c++11 -Igoogletest -Igoogletest/include -pthread -c googletest/src/gtest-all.cc -o gtest-all.o; cd ..
	$(AR) -rv libgtest.a gtest/gtest-all.o
	mkdir -p /tmp/gtest/include /tmp/gtest/lib
	cp -r gtest/googletest/include/gtest /tmp/gtest/include
	mv libgtest.a /tmp/gtest/lib
	rm -rf release-1.10.0.zip

gtest: | /tmp/gtest/include/gtest
