import subprocess
import sys


def run_command(command):
    try:
        subprocess.check_output(command, stderr=subprocess.STDOUT, shell=True)
    except subprocess.CalledProcessError as e:
        print(f"Error executing command: {e.output}")
        sys.exit(1)


def main(tag_name):
    # Example of a non-CI commit. Adjust as needed.
    run_command("git commit --allow-empty -m 'Triggering release process'")
    run_command(f"git tag {tag_name}")
    run_command("git push origin --tags")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python script.py <tag-name>")
        sys.exit(1)
    tag_name = sys.argv[1]
    main(tag_name)
