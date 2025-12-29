Model based approach for loading fltk tables.

Try:
```
cargo run --example people
```

![image](https://user-images.githubusercontent.com/1972001/155335256-203ca0be-a9df-4283-b8ec-a04177fbe4c4.png)

TODO:
* clip header (paints outside of area)
* default implementations of column based SimpleTableModel functions
* Prettier (internal white space)
* replace optional cell results with enum (WIDGET(widget),TEXT(Align,wrap)). Should remove DrawDelegate?
* fix widget actions (currently clicking on row will actvate button)