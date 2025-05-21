from setuptools import setup

setup(
    name='mcp',
    version='0.1.0',
    py_modules=['cli'],
    entry_points={
        'console_scripts': [
            'mcp = cli:main',
        ],
    },
)
