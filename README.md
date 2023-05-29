# Create keyboard layouts

An app to help layout keys based on your corpus and preferences

You'll need to install Rust to build this.

Run it with:
```
> cargo run --release -- <name of corpus text files>
```

Pass the text files you want to use as the corpus to solve the keyboard. It will first read the corpus, then build the runs, and then try to solve for the best layout.

Ones it reaches a local optimum for some time, you can safely stop it and try what it's found.

You can also run with:
```
> cargo run --release -- <name of corpus text files> --debug
```

To have it process the corpus, then output the counts of the runs it found.

In the source, at the top, you can also change the scoring for the various types of movements. It's pretty straightforward to change them or add new ones.
