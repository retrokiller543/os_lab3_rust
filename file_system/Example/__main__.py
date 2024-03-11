from RusticFS import setup_file_logger

from Shell import Shell

if __name__ == "__main__":
    setup_file_logger("trace")
    shell = Shell()
    shell.run()
