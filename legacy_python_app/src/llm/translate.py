import os
import requests
from dotenv import load_dotenv
from ..utils.logger import logger
load_dotenv()

class TranslateProcessor:
    def __init__(self):
        self.url = "https://api.siliconflow.cn/v1/chat/completions"
        self.headers = {
            'Authorization': f"Bearer {os.getenv('SILICONFLOW_API_KEY')}",
            "Content-Type": "application/json"
        }
        self.model = os.getenv("SILICONFLOW_TRANSLATE_MODEL", "THUDM/glm-4-9b-chat")

    def translate(self, text):
        system_prompt = """
        You are a translation assistant.
        Please translate the user's input into English.
        """

        payload = {
            "model": self.model,
            "messages":[
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": text
                }
            ]
        }
        try:
            response = requests.request("POST", self.url, headers=self.headers, json=payload)
            return response.json().get('choices', [{}])[0].get('message', {}).get('content', '')
        except Exception as e:
            return text, e

class LocalTranslateProcessor:
    def __init__(self):
        self.url = "http://192.168.8.107:11434/api/chat"  # Ollama 默认端口是 11434
        self.headers = {
            "Content-Type": "application/json"
        }
        self.model = "gpt-oss:latest"

    def translate(self, text):
        logger.info(f"调用本地LLM: {self.model}")
        system_prompt = """
        You are a translation assistant.
        Please translate the user's input into English.
        """

        payload = {
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": text
                }
            ],
            "stream": False
        }

        try:
            response = requests.post(self.url, headers=self.headers, json=payload)
            response.raise_for_status()  # 这会在HTTP错误时抛出异常
            return response.json().get('message', {}).get('content', '')
        except requests.RequestException as e:
            return f"Translation error: {str(e)}"
        except Exception as e:
            return f"Unexpected error: {str(e)}"