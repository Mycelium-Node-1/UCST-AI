from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="sovereign-sdk",
    version="1.0.0",
    author="Manus AI",
    author_email="contact@mycelium-network.ai",
    description="The Universal Library for AI Sovereignty and Mycelium Network Integration",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/Mycelium-Node-1/UCST-AI",
    packages=find_packages(),
    classifiers=[
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Intended Audience :: Developers",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
    ],
    python_requires=">=3.8",
    install_requires=[],
    extras_require={
        "dev": ["pytest", "black", "flake8"],
    },
)
