I have tried to write a parser using `winnow`  and am not haning success. I need help. Could you write code - using winnow parsers - that parses the following patterns:

1) Input: "This is some text that //has italics// in the middle."

This should  result in a Vec<Inline> with the following entries:
    Inline::PlainText{text: "This is some text that "}
    Inline::Italics{text: "has italics"}
    Inline::PlainText{text: " in the middle."}


2) Input: "Text that has ''bold text'' in it."

This should  result in a Vec<Inline> with the following entries:
    Inline::PlainText{text: "Text that has "}
    Inline::Bold{text: "bold text"}
    Inline::PlainText{text: " in it."}

3) Input: "Some text with a mixture of //italicised text// and also ''some bold text'' in it".  

This should  result in a Vec<Inline> with the following entries:
    Inline::PlainText{text: "Some text with a mixture of "}
    Inline::Italics{text: "italicised text"}
    Inline::PlainText{text: " and also "}
    Inline::Bold{text: "some bold text"}
    Inline::PlainText{text: " in it."} 







    <wiki.text> := <inline>* eof