FROM python:3.9

WORKDIR /app
COPY requirements.txt /app/requirements.txt
COPY rand_messages.py /app/rand_messages.py
ENV PYTHONUNBUFFERED=1
RUN pip install -r /app/requirements.txt

CMD [ "python","/app/rand_messages.py" ]