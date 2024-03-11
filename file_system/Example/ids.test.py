def fibonacci(n):
    """
    This function calculates the nth Fibonacci number using recursion.
    """
    if n <= 1:
        return n
    else:
        return fibonacci(n-1) + fibonacci(n-2)
def main():
    """
    This function defines the entry point for the script.
    It calculates and prints the first 20 Fibonacci numbers.
    """
    for i in range(20):
        result = fibonacci(i)
        print(f"Fibonacci number {i} is {result}")
if __name__ == "__main__":
    main()

