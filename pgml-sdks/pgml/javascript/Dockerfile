FROM quay.io/pypa/manylinux2014_x86_64

# Install node and npm
RUN yum install -y nodejs
RUN yum install -y npm

# Gives build errors if we don't have this
RUN yum install -y perl-IPC-Cmd

# Create a new user. We need this or we run as root and this will cause permission issues
RUN groupadd --g 1000 groupcontainer
RUN useradd -u 1000 -G groupcontainer -m containeruser
USER containeruser

# Install cargo
RUN curl --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add cargo to path
ENV PATH /home/containeruser/.cargo/bin:$PATH

ENTRYPOINT ["npm", "run"]
