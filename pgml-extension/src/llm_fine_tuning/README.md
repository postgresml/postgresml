# Standalone Python scripts for Large Language Model Fine Tuning

## Pre-requisites
- Python3
- Python3-pip

**for GPU**
- NVIDIA drivers compatible with your GPU. 

Use these instructions to install drivers and Python3-pip

```
sudo apt update
sudo apt install ubuntu-drivers-common
ubuntu-drivers devices
# Find the driver necessary for that specific GPU .. replace XXX with the *latest* driver in the list
sudo apt install nvidia-driver-XXX
#sudo add-apt-repository ppa:graphics-drivers/ppa --yes
sudo reboot
sudo apt install python3-pip
```
## Usage
- `python3 -m venv venv`
- `source venv/bin/activate`
- `pip install -r requirements.txt`

### Training

**GPUs**

*Note torchrun and not python to run training*

`torchrun train.py netflix_titles_small.csv description --model_name gpt2 --tokenizer_name gpt2 --batch_size 4 --get_gpu_utilization True`

**CPUs**
`python train.py netflix_titles_small.csv description --model_name gpt2 --tokenizer_name gpt2 --batch_size 4`


**Help**

Below command will show all the options available

`python train.py --help`

### Inference

`python generate.py <prompt>`

**Help**

Below command will show all the options available

`python generate.py --help`




