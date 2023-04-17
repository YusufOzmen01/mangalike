# mangalike

Mangalike is a command line utility for you to download and synchronize your manga library. Another bonus is you can export all of your mangas in epub format so that you can use them in e reader applications!

App is still WIP so bugs are expected. You can create an issue or a pull request if you encounter one.

## Usage

```
./mangalike 
    -s, --sync   // Sync your mangas
    -c, --create // Create a library in the folder you're in
    -e, --export // Export all of your mangas in exports folder
```

## mangalike.toml
If you want to only synchronize mangas after a certain chapter, you can add them to the mangalike.toml file in your library in `MANGA_ID=STARTING_CHAPTER` format like below:
```
to970571=146
qm951521=76.5
mv989778=14
hl984994=36
```

## Contribute
Any contribution is welcome! I'm still learning rust so if I can learn something I'm open to it