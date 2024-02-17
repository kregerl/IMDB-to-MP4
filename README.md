# IMDB to MP4
**This program requires NodeJS**

This program is the product of reverse engineering  the processes vidsrc.me uses to watch a given movie or show with only the IMDB url. 

Given an IMDB URL, this program will download the movie or all episodes and seasons of the show.

For example the Deadpool IMDB url is `https://www.imdb.com/title/tt1431045/`

When downloading shows, directories are created for each season of of the show. Episodes are downloaded into their respective directories.  
For example:
```
<Show Name>
    - Season 01
        - <Show Name> S01E01
```

## Usage
```
Usage: movie-downloader [OPTIONS]

Options:
  -u, --from-url <from_url>    
  -f, --from-file <from_file>  
  -h, --help                   Print help
```

`--from-url` can be used to provide a single imdb url to download a movie/show from

`--from-file` can be used to provide a yaml file of IMDB urls to get movies/shows from.
An example YAML file is shown below.
```yaml
urls:
  - "https://www.imdb.com/title/tt1431045"
```

