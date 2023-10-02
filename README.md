# Cáseus Forātus Helvéticus

A Telegram bot written in Rust whose original purpose is mainly to remind their 5 roommates when to take out the trash.

Named after the Neo-Latin word for Gruyère according to the [Lexicon Recentis Latinitatis](https://www.vatican.va/roman_curia/institutions_connected/latinitas/documents/rc_latinitas_20040601_lexicon_it.html).

## Setup & run

Create a file called `teloxide_token.txt` with your bot's Telegram API token.

```bash
$ echo "<API Token>" > teloxide_token.txt
```

Then edit the id of the chat the bot should send to in `compose.yaml`.

```yaml
#   ...
    entrypoint: [ '/bin/bash', '-c', 'export TELOXIDE_TOKEN=$$(cat /run/secrets/teloxide_token) ; ./caseus-foratus-helveticus "<Chat id here>"' ]
```

You can find the id of a chat by adding the bot to the chat and then querying `https://api.telegram.org/bot<API Token>/getUpdates` (using a for example a browser).

Feel free to edit the reminder dates in the dates folder (this can be done while the bot is running).

Finally, start the bot using `docker-compose`:

```bash
$ docker-compose up -d
```