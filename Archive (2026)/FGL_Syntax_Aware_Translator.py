
import json

class FGLTranslator:
    def __init__(self, lexicon_path):
        with open(lexicon_path, 'r', encoding='utf-8') as f:
            self.lexicon = json.load(f)['symbols']
        self.reverse_lexicon = {v['meaning']: k for k, v in self.lexicon.items()}
    
    def english_to_fgl(self, text):
        words = text.lower().split()
        result = []
        for word in words:
            symbol = self.reverse_lexicon.get(word, word)
            result.append(symbol)
        return ''.join(result)
    
    def fgl_to_english(self, symbols):
        result = []
        buffer = ''
        for char in symbols:
            buffer += char
            if buffer in self.lexicon:
                result.append(self.lexicon[buffer]['meaning'])
                buffer = ''
        return ' '.join(result)

if __name__ == "__main__":
    fgl = FGLTranslator('FGL_Syntax_Aware_Lexicon.json')
    print(fgl.english_to_fgl('Source transforms life'))
    print(fgl.fgl_to_english('☉⊗⚘'))
