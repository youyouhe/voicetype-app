import os
import threading
import time
from functools import wraps

import dotenv
import httpx

from src.llm.translate import TranslateProcessor,LocalTranslateProcessor
from ..utils.logger import logger

dotenv.load_dotenv()

def timeout_decorator(seconds):
    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            result = [None]
            error = [None]
            completed = threading.Event()

            def target():
                try:
                    result[0] = func(*args, **kwargs)
                except Exception as e:
                    error[0] = e
                finally:
                    completed.set()

            thread = threading.Thread(target=target)
            thread.daemon = True
            thread.start()

            if completed.wait(seconds):
                if error[0] is not None:
                    raise error[0]
                return result[0]
            raise TimeoutError(f"操作超时 ({seconds}秒)")

        return wrapper
    return decorator

class SenseVoiceSmallProcessor:
    # 类级别的配置参数
    DEFAULT_TIMEOUT = 20  # API 超时时间（秒）
    DEFAULT_MODEL = "FunAudioLLM/SenseVoiceSmall"
    
    def __init__(self):
        api_key = os.getenv("SILICONFLOW_API_KEY")
        assert api_key, "未设置 SILICONFLOW_API_KEY 环境变量"
        
        self.convert_to_simplified = os.getenv("CONVERT_TO_SIMPLIFIED", "false").lower() == "true"
        # self.cc = OpenCC('t2s') if self.convert_to_simplified else None
        # self.symbol = SymbolProcessor()
        # self.add_symbol = os.getenv("ADD_SYMBOL", "false").lower() == "true"
        # self.optimize_result = os.getenv("OPTIMIZE_RESULT", "false").lower() == "true"
        self.timeout_seconds = self.DEFAULT_TIMEOUT
        self.translate_processor = LocalTranslateProcessor()
        #self.translate_processor = TranslateProcessor()

    def _convert_traditional_to_simplified(self, text):
        """将繁体中文转换为简体中文"""
        if not self.convert_to_simplified or not text:
            return text
        return self.cc.convert(text)

    @timeout_decorator(10)
    def _call_api(self, audio_data):
        """调用硅流 API"""
        transcription_url = "https://api.siliconflow.cn/v1/audio/transcriptions"
        
        files = {
            'file': ('audio.wav', audio_data),
            'model': (None, self.DEFAULT_MODEL)
        }

        headers = {
            'Authorization': f"Bearer {os.getenv('SILICONFLOW_API_KEY')}"
        }

        with httpx.Client() as client:
            response = client.post(transcription_url, files=files, headers=headers)
            response.raise_for_status()
            return response.json().get('text', '获取失败')


    def process_audio(self, audio_buffer, mode="transcriptions", prompt=""):
        """处理音频（转录或翻译）
        
        Args:
            audio_buffer: 音频数据缓冲
            mode: 'transcriptions' 或 'translations'，决定是转录还是翻译
        
        Returns:
            tuple: (结果文本, 错误信息)
            - 如果成功，错误信息为 None
            - 如果失败，结果文本为 None
        """
        try:
            start_time = time.time()
            
            logger.info(f"正在调用 硅基流动 API... (模式: {mode})")
            result = self._call_api(audio_buffer)

            logger.info(f"API 调用成功 ({mode}), 耗时: {time.time() - start_time:.1f}秒")
            # result = self._convert_traditional_to_simplified(result)
            if mode == "translations":
                result = self.translate_processor.translate(result)
            logger.info(f"识别结果: {result}")
            
            # if self.add_symbol:
            #     result = self.symbol.add_symbol(result)
            #     logger.info(f"添加标点符号: {result}")
            # if self.optimize_result:
            #     result = self.symbol.optimize_result(result)
            #     logger.info(f"优化结果: {result}")

            return result, None

        except TimeoutError:
            error_msg = f"❌ API 请求超时 ({self.timeout_seconds}秒)"
            logger.error(error_msg)
            return None, error_msg
        except Exception as e:
            error_msg = f"❌ {str(e)}"
            logger.error(f"音频处理错误: {str(e)}", exc_info=True)
            return None, error_msg
        finally:
            audio_buffer.close()  # 显式关闭字节流

class LocalASRProcessor:
    DEFAULT_TIMEOUT = 20  # API 超时时间（秒）
    DEFAULT_ASR_URL = "http://192.168.8.107:5001/inference"
    
    def __init__(self):
        self.asr_url = os.getenv("LOCAL_ASR_URL", self.DEFAULT_ASR_URL)
        self.asr_key = os.getenv("LOCAL_ASR_KEY", "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456")  # 默认key，可以通过环境变量覆盖
        self.timeout_seconds = self.DEFAULT_TIMEOUT
        self.translate_processor = LocalTranslateProcessor()

    @timeout_decorator(10)
    def _call_api(self, audio_data, lang="auto"):
        """调用本地 ASR API"""
        files = {
            'file': ('audio.wav', audio_data, 'audio/wav')
        }
        data = {
            'response_format': 'srt',
            'language': lang
        }

        # 1. 创建 headers 字典
        headers = {
            #'Authorization': f'Bearer {self.asr_key}'
            # 如果API要求的不是Bearer token，而是其他自定义header，
            # 可以替换成类似下面的形式：
            'X-API-KEY': self.asr_key
        }

        with httpx.Client() as client:
            # 2. 在 post 请求中传入 headers
            response = client.post(self.asr_url, files=files, data=data, headers=headers)
            response.raise_for_status()
            # 当 response_format="srt" 时，直接返回文本内容
            content_type = response.headers.get('content-type', '')
            if 'application/json' in content_type:
                return response.json()
            else:
                return response.text
            
    def process_audio(self, audio_buffer, mode="transcriptions", prompt=""):
        """处理音频（转录或翻译）"""
        try:
            start_time = time.time()
            
            logger.info(f"正在调用本地 ASR API... (模式: {mode})")
            result = self._call_api(audio_buffer)

            # 处理 API 返回的结果
            # 新的响应格式: {"code":0,"msg":"ok","data":"1\n00:00:00,000 --> 00:00:01,980\n时间就是金钱,我的朋友。"}
            if isinstance(result, str):
                transcription = result
                raw_text = result
                clean_text = result
            elif isinstance(result, dict):
                if 'data' in result:
                    # 新的响应格式
                    transcription = result['data']
                    raw_text = result['data']
                    clean_text = result['data']
                elif 'result' in result and result['result']:
                    # 旧的响应格式
                    transcription = result['result'][0].get('text', '获取失败')
                    raw_text = result['result'][0].get('raw_text', '')
                    clean_text = result['result'][0].get('clean_text', '')
                else:
                    transcription = '获取失败'
                    raw_text = ''
                    clean_text = ''
            else:
                transcription = '获取失败'
                raw_text = ''
                clean_text = ''

            logger.info(f"API 调用成功 ({mode}), 耗时: {time.time() - start_time:.1f}秒")
            logger.info(f"原始识别结果: {raw_text}")
            logger.info(f"清理后的识别结果: {clean_text}")
            
            if mode == "translations":
                translated_text = self.translate_processor.translate(transcription)
                logger.info(f"翻译结果: {translated_text}")
                return translated_text, None
            else:
                logger.info(f"识别结果: {transcription}")
                return transcription, None

        except TimeoutError:
            error_msg = f"❌ API 请求超时 ({self.timeout_seconds}秒)"
            logger.error(error_msg)
            return None, error_msg
        except Exception as e:
            error_msg = f"❌ {str(e)}"
            logger.error(f"音频处理错误: {str(e)}", exc_info=True)
            return None, error_msg
        finally:
            audio_buffer.close()  # 显式关闭字节流