# setup.py
from setuptools import setup

setup(
    name='aigpt-mcp',
    py_modules=['cli'],
    entry_points={
        'console_scripts': [
            'mcp = cli:main',
        ],
    },
)
