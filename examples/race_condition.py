import threading
import time

# A shared global variable
count = 0

def increment():
    global count
    for _ in range(50):
        # Read-Modify-Write: A common source of race conditions
        tmp = count
        time.sleep(0.01) # Intentionally force a context switch
        count = tmp + 1

print("Running Python (Threads)...")
t1 = threading.Thread(target=increment)
t2 = threading.Thread(target=increment)

t1.start()
t2.start()

t1.join()
t2.join()

# In Python, this will likely be much less than 100
print(f"Python Final Count (Expected 100): {count}")
