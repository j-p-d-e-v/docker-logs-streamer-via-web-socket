from faker import Faker
import time
import logging
import sys
import os
logger = logging.getLogger(__name__)
logger.setLevel(logging.DEBUG)
ch = logging.StreamHandler()
ch.setLevel(logging.DEBUG)
logger.addHandler(ch)

def main():
    sleep_time = float(os.environ.get("SLEEP_TIME",1))
    loop_limit = int(os.environ.get("LOOP_LIMIT",0))
    fake = Faker()
    counter = 0
    while True:
        logger.info(fake.sentence())
        time.sleep(sleep_time)
        if loop_limit > 0:
            counter += 1
            if counter > loop_limit:
                break
    print("DONE!")

if __name__ == "__main__":
    main()