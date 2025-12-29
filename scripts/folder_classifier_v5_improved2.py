#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
文件夹智能分类器 V5 - 责任链模式重构版（结构优化）

核心模块:
1. Data Structures (数据结构): 定义上下文、结果和任务。
2. AhoCorasick (AC自动机): 关键词匹配核心。
3. Chain of Responsibility (责任链): 定义处理器基类和具体的处理步骤。
4. Threading Workers (线程工作者): 分类计算和IO操作的分离。
5. Main Classifier (主分类器): 协调配置、构建责任链和管理多线程。
"""

# ----------------------------------------------------------------------
# 1. 导入和日志配置
# ----------------------------------------------------------------------
import warnings
warnings.filterwarnings("ignore", category=UserWarning, module="zhconv")

import argparse
import json
import logging
import re
import threading
import time
from abc import ABC, abstractmethod
from collections import deque
from dataclasses import dataclass
from pathlib import Path
from queue import Queue
from typing import Dict, List, Optional, Set, Tuple, Any
import shutil

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)


# ----------------------------------------------------------------------
# 2. Data Structures (数据结构)
# ----------------------------------------------------------------------

@dataclass
class ClassificationContext:
    """分类上下文，在责任链中传递"""
    folder_name: str
    folder_path: Path
    categories: List[str]  # 可能的分类列表
    scores: Dict[str, float]  # 分类分数
    metadata: Dict[str, Any]  # 元数据
    
    def __init__(self, folder_name: str, folder_path: Path):
        self.folder_name = folder_name
        self.folder_path = folder_path
        self.categories = []
        self.scores = {}
        self.metadata = {}


@dataclass
class ClassificationResult:
    """分类结果"""
    status: str  # 'classified', 'unclassified', 'error', 'pending'
    category: Optional[str] = None
    categories: Optional[List[str]] = None
    folder_name: Optional[str] = None
    error_message: Optional[str] = None
    score: Optional[float] = None
    scores_detail: Optional[Dict[str, float]] = None


@dataclass
class MoveTask:
    """文件移动任务"""
    source_path: Path
    target_path: Path
    folder_name: str
    category: str


# ----------------------------------------------------------------------
# 3. AhoCorasick (AC自动机)
# ----------------------------------------------------------------------

class AhoCorasickNode:
    """AC自动机节点"""
    def __init__(self):
        self.children: Dict[str, AhoCorasickNode] = {}
        self.fail: Optional[AhoCorasickNode] = None
        # output: (category, original_keyword, score)
        self.output: List[Tuple[str, str, float]] = [] 
        self.is_end: bool = False


class AhoCorasickMatcher:
    """AC自动机关键词匹配器"""
    
    def __init__(self):
        self.root = AhoCorasickNode()
        self.keywords: Set[str] = set()
        self.category_config = {}  # 用于分数计算查找组合关键词

    def add_keyword(self, keyword: str, category: str, score: float = 1.0):
        """
        添加关键词到AC自动机。
        """
        if not keyword:
            return
            
        original_keyword = keyword
        keyword = keyword.lower().strip()
        
        # 只添加原始关键词，不再处理中文拼音转换
        self.keywords.add(keyword)
        current = self.root
        for char in keyword:
            if char not in current.children:
                current.children[char] = AhoCorasickNode()
            current = current.children[char]
        current.is_end = True
        current.output.append((category, original_keyword, score))
    
    def build_failure_links(self):
        """构建失败指针"""
        queue = deque()
        
        for node in self.root.children.values():
            node.fail = self.root
            queue.append(node)
        
        while queue:
            current_node = queue.popleft()
            
            for char, child_node in current_node.children.items():
                queue.append(child_node)
                
                fail_node = current_node.fail
                while fail_node and char not in fail_node.children:
                    fail_node = fail_node.fail
                
                child_node.fail = fail_node.children.get(char, self.root) if fail_node else self.root
                
                if child_node.fail:
                    child_node.output.extend(child_node.fail.output)
    
    def find_matches(self, text: str) -> List[Tuple[str, str, float, int, int]]:
        """
        查找匹配的关键词，并记录位置信息
        
        返回格式: [(category, original_keyword, score, start_pos, end_pos), ...]
        其中 start_pos 是关键词在文本中的起始位置，end_pos 是结束位置
        """
        if not text:
            return []
        
        text = text.lower()
        results = []
        current = self.root
        current_pos = 0
        
        for char in text:
            # 失败指针回溯
            while current and char not in current.children:
                current = current.fail
            
            current = current.children.get(char, self.root) if current else self.root
            current_pos += 1
            
            # 收集输出，记录位置信息
            if current and current.output:
                for category, keyword, score in current.output:
                    # 计算匹配的起始位置和结束位置
                    end_pos = current_pos
                    start_pos = end_pos - len(keyword.lower())
                    results.append((category, keyword, score, start_pos, end_pos))
        
        return results


# ----------------------------------------------------------------------
# 4. Chain of Responsibility (责任链)
# ----------------------------------------------------------------------

class Handler(ABC):
    """责任链处理器基类"""
    
    def __init__(self):
        self.next_handler: Optional[Handler] = None
    
    def set_next(self, handler: 'Handler') -> 'Handler':
        """设置下一个处理器"""
        self.next_handler = handler
        return handler
    
    @abstractmethod
    def handle(self, context: ClassificationContext) -> bool:
        """处理请求"""
        pass
    
    def _call_next(self, context: ClassificationContext) -> bool:
        """调用下一个处理器"""
        if self.next_handler:
            return self.next_handler.handle(context)
        return True


class PreprocessingHandler(Handler):
    """1. 预处理处理器 - 繁简转换、构建拼音组合关键字、英文小写和空格处理"""
    
    def __init__(self):
        super().__init__()
        self.zhconv_available = False
        self.traditional_to_simplified = self._init_conversion()
        self.pypinyin_available = False
        self._check_pypinyin()

    def _init_conversion(self):
        """初始化简繁转换工具"""
        try:
            import zhconv
            self.zhconv = zhconv
            self.zhconv_available = True
            logging.info("已成功导入zhconv库，将使用更全面的简繁转换")
            return None # 使用zhconv，不需要fallback字典
        except ImportError:
            # 简繁转换映射表 - 仅作为fallback
            logging.info("未找到zhconv库，将使用内置fallback字典进行简繁转换")
            return {
                '發': '发', '現': '现', '時': '时', '間': '间', '個': '个',
                '來': '来', '對': '对', '說': '说', '會': '会', '過': '过',
                '為': '为', '這': '这', '們': '们', '開': '开', '關': '关',
                '長': '长', '電': '电', '腦': '脑', '軟': '软', '體': '体',
                '檔': '档', '案': '案', '夾': '夹', '資': '资', '料': '料',
                '圖': '图', '書': '书', '視': '视', '頻': '频', '音': '音',
                '樂': '乐', '遊': '游', '戲': '戏', '網': '网', '頁': '页',
                '論': '论', '壇': '坛', '課': '课', '程': '程', '學': '学',
                '習': '习', '練': '练', '習': '习', '測': '测', '試': '试',
                '驗': '验', '證': '证', '碼': '码', '密': '密', '碼': '码',
                '帳': '账', '號': '号', '註': '注', '冊': '册', '登': '登',
                '錄': '录', '記': '记', '錄': '录', '憶': '忆', '憶': '忆',
                '語': '语', '言': '言', '漢': '汉', '字': '字', '詞': '词',
                '語': '语', '句': '句', '話': '话', '語': '语', '音': '音',
                '聲': '声', '調': '调', '節': '节', '奏': '奏', '樂': '乐',
                '譜': '谱', '歌': '歌', '詞': '词', '曲': '曲', '調': '调',
                '澤': '泽', '鴿': '鸽'
            }
    
    def _check_pypinyin(self):
        """检查并导入 pypinyin 库，用于中文转拼音"""
        try:
            from pypinyin import lazy_pinyin, Style
            self.pypinyin_available = True
            self.lazy_pinyin = lazy_pinyin
            self.Style = Style
            logging.info("已成功导入pypinyin库，将支持拼音组合关键字构建")
        except ImportError:
            self.pypinyin_available = False
            logging.warning("未找到pypinyin库，将不支持拼音组合关键字构建")
    
    def _is_chinese(self, char: str) -> bool:
        """判断字符是否为中文"""
        return '\u4e00' <= char <= '\u9fff'
    
    def _is_all_chinese(self, text: str) -> bool:
        """判断文本是否全为中文"""
        return all(self._is_chinese(char) for char in text if char.strip())
    
    def _convert_chinese_to_pinyin(self, text: str) -> List[str]:
        """将中文文本转换为拼音数组"""
        if not text or not self.pypinyin_available:
            return []
        
        # 将中文拆分为单字，生成对应的拼音数组
        pinyin_list = []
        for char in text:
            if self._is_chinese(char):
                # 获取单字拼音
                pinyin = self.lazy_pinyin(char, style=self.Style.NORMAL)[0].lower()
                pinyin_list.append(pinyin)
        return pinyin_list
    
    def _traditional_to_simplified(self, text: str) -> str:
        """繁体转简体"""
        if self.zhconv_available:
            return self.zhconv.convert(text, 'zh-cn')
        else:
            result = ""
            for char in text:
                result += self.traditional_to_simplified.get(char, char)
            return result
    
    def _build_pinyin_combo_keywords(self, keywords: List[Any]) -> List[Any]:
        """
        构建拼音组合关键字
        
        规则：
        1. 对于中文普通关键字（字符串类型），将其拆分为单字，为每个单字生成对应的拼音，然后创建由这些拼音组成的数组作为组合关键字
        2. 对于包含中文的组合关键字（数组类型），若数组中包含中文元素，将中文元素转换为对应的拼音数组，生成包含拼音元素的新组合关键字数组
        
        Args:
            keywords: 原始关键字列表
            
        Returns:
            扩展后的关键字列表，包含原始关键字和生成的拼音组合关键字
        """
        if not keywords or not self.pypinyin_available:
            return keywords
        
        extended_keywords = keywords.copy()
        
        for keyword in keywords:
            if isinstance(keyword, str) and self._is_all_chinese(keyword):
                # 中文普通关键字：生成拼音组合关键字
                pinyin_list = self._convert_chinese_to_pinyin(keyword)
                if pinyin_list:
                    extended_keywords.append(pinyin_list)
            elif isinstance(keyword, list):
                # 组合关键字：检查是否包含中文元素
                has_chinese = any(isinstance(item, str) and self._is_all_chinese(item) for item in keyword)
                if has_chinese:
                    # 生成包含拼音的新组合关键字
                    new_combo = []
                    for item in keyword:
                        if isinstance(item, str) and self._is_all_chinese(item):
                            # 中文元素：转换为拼音数组
                            pinyin_list = self._convert_chinese_to_pinyin(item)
                            new_combo.extend(pinyin_list)
                        else:
                            # 非中文元素：保持原样
                            new_combo.append(item)
                    if new_combo:
                        extended_keywords.append(new_combo)
        
        return extended_keywords
    
    def handle(self, context: ClassificationContext) -> bool:
        """预处理文件夹名称"""
        try:
            folder_name = context.folder_name
            
            # 1. 繁简转换
            folder_name = self._traditional_to_simplified(folder_name)
            
            # 2. 构建拼音组合关键字 - 此步骤需要访问分类配置，实际应在初始化时处理
            # 此处保留接口，具体实现移至FolderClassifierV5Refactored类中
            
            # 3. 英文小写转换
            folder_name = folder_name.lower()
            
            # 4. 移除非字母字符间的空格 (例如 "秀 人 网" -> "秀人网")
            folder_name = re.sub(r'(?<![a-z])\s+(?![a-z])', '', folder_name)
            
            context.metadata['preprocessed_name'] = folder_name
            
            return self._call_next(context)
            
        except Exception as e:
            logging.error(f"预处理失败: {e}")
            return False


class KeywordMatchingHandler(Handler):
    """2. 关键词匹配处理器 - 使用AC自动机"""
    
    def __init__(self, ac_matcher: AhoCorasickMatcher):
        super().__init__()
        self.ac_matcher = ac_matcher
    
    def handle(self, context: ClassificationContext) -> bool:
        """执行关键词匹配"""
        try:
            preprocessed_name = context.metadata.get('preprocessed_name', context.folder_name)
            
            # 查找匹配: (category, original_keyword, score, start_pos, end_pos)
            matches = self.ac_matcher.find_matches(preprocessed_name)
            
            # 按分类分组，并保存所有匹配信息，包括位置
            category_matches = {}
            for category, keyword, score, start_pos, end_pos in matches:
                if category not in category_matches:
                    category_matches[category] = []
                
                # 保存所有匹配，包括位置信息: (keyword, score, start_pos, end_pos)
                category_matches[category].append((keyword, score, start_pos, end_pos))
            
            context.metadata['keyword_matches'] = category_matches
            context.categories = list(category_matches.keys())
            
            return self._call_next(context)
            
        except Exception as e:
            logging.error(f"关键词匹配失败: {e}")
            return False


class ExclusionFilterHandler(Handler):
    """3. 排除过滤处理器"""
    
    def __init__(self, exclude_keywords: Any):
        super().__init__()
        self.exclude_keywords = exclude_keywords
    
    def _should_exclude(self, context: ClassificationContext, category: str) -> bool:
        """判断是否应该排除该分类"""
        try:
            # 获取该分类的排除规则
            exclude_rules = []
            if isinstance(self.exclude_keywords, dict):
                exclude_rules = self.exclude_keywords.get(category, [])
            else:
                # 如果排除规则不是字典，跳过排除过滤
                logging.debug(f"排除规则格式不支持，跳过分类 {category} 的排除过滤")
                return False
            
            folder_name = context.metadata.get('preprocessed_name', context.folder_name)
            folder_name_lower = folder_name.lower()
            
            for rule in exclude_rules:
                if isinstance(rule, dict) and 'keywords' in rule:
                    # 组合排除关键词 - 全部匹配才排除
                    combo_keywords = rule['keywords']
                    if isinstance(combo_keywords, list):
                        if all(keyword.lower() in folder_name_lower for keyword in combo_keywords):
                            return True
                else:
                    # 普通排除关键词
                    if str(rule).lower() in folder_name_lower:
                        return True
            
            return False
        except Exception as e:
            logging.error(f"判断排除规则时出错 (分类: {category}): {e}")
            return False
    
    def handle(self, context: ClassificationContext) -> bool:
        """应用排除过滤"""
        try:
            # 过滤掉被排除的分类
            filtered_categories = []
            
            for category in context.categories:
                if not self._should_exclude(context, category):
                    filtered_categories.append(category)
                else:
                    logging.debug(f"分类 {category} 被排除规则过滤")
            
            # 更新分类列表
            context.categories = filtered_categories
            
            return self._call_next(context)
            
        except Exception as e:
            logging.error(f"排除过滤失败: {e}")
            return False


class ScoreCalculationHandler(Handler):
    """4. 分数计算处理器 - 实现组合关键词逻辑"""
    
    def __init__(self, category_config: Any):
        super().__init__()
        self.category_config = category_config
    
    def _calculate_category_score(self, category: str, matches: List[Tuple[str, float, int, int]]) -> float:
        """计算分类分数 - 实现普通关键词和组合关键词逻辑"""
        config = {}
        if isinstance(self.category_config, dict):
            # 字典格式：{"category_name": {"keywords": [...]}}
            config = self.category_config.get(category, {})
        elif isinstance(self.category_config, list):
            # 列表格式：[{"name": "category_name", "keywords": [...]}]
            for cat_config in self.category_config:
                if cat_config.get('name') == category:
                    config = cat_config
                    break
        keywords = config.get('keywords', [])
        priority = config.get('priority', 1)
        
        base_score = 0.0
        
        # 1. 预处理匹配结果，构建关键词到位置的映射
        # 格式: {keyword: [(start_pos, end_pos), ...], ...}
        keyword_positions = {}
        for kw, score, start_pos, end_pos in matches:
            kw_lower = kw.lower()
            if kw_lower not in keyword_positions:
                keyword_positions[kw_lower] = []
            # 只添加不重复的位置
            if (start_pos, end_pos) not in keyword_positions[kw_lower]:
                keyword_positions[kw_lower].append((start_pos, end_pos))
        
        # 2. 处理每个关键词条目
        for keyword_entry in keywords:
            if isinstance(keyword_entry, (dict, list)):
                # 组合关键词 (字典或列表格式)
                combo_keywords = keyword_entry.get('keywords') if isinstance(keyword_entry, dict) else keyword_entry
                
                if isinstance(combo_keywords, list) and combo_keywords:
                    # 处理组合关键字
                    combo_keywords_lower = [str(kw).lower() for kw in combo_keywords]
                    
                    # 检查所有子关键字是否都存在
                    all_keywords_present = True
                    for kw in combo_keywords_lower:
                        if kw not in keyword_positions:
                            all_keywords_present = False
                            break
                    
                    if not all_keywords_present:
                        continue
                    
                    # 检查是否为相同子关键字的组合
                    is_same_keywords = all(kw == combo_keywords_lower[0] for kw in combo_keywords_lower)
                    
                    if is_same_keywords:
                        # 3. 相同子关键字组合的处理逻辑
                        # 例如: ["le", "le"] 或 ["le", "le", "le"]
                        kw = combo_keywords_lower[0]
                        positions = keyword_positions[kw]
                        
                        # 排序位置，确保按出现顺序处理
                        positions_sorted = sorted(positions, key=lambda x: x[0])
                        
                        # 检查是否有足够的位置匹配
                        if len(positions_sorted) < len(combo_keywords_lower):
                            continue
                        
                        # 检查是否存在满足距离条件的连续匹配
                        has_valid_match = False
                        
                        # 对于n个相同子关键字，需要检查 (len(positions) - n + 1) 种可能的连续匹配
                        for i in range(len(positions_sorted) - len(combo_keywords_lower) + 1):
                            valid = True
                            
                            # 检查每对相邻匹配位置之间的距离
                            for j in range(len(combo_keywords_lower) - 1):
                                # 获取当前匹配和下一个匹配的位置
                                current_start, current_end = positions_sorted[i + j]
                                next_start, next_end = positions_sorted[i + j + 1]
                                
                                # 计算位置差值减去子关键字长度
                                distance = next_start - current_end
                                keyword_length = len(kw)
                                
                                # 根据需求：距离差值减去子关键字长度必须小于4
                                if (distance - keyword_length) >= 4:
                                    valid = False
                                    break
                            
                            if valid:
                                has_valid_match = True
                                break
                        
                        if has_valid_match:
                            base_score += 1.0
                    else:
                        # 4. 不同子关键字组合的处理逻辑
                        # 只需要各子关键字按顺序出现即可，不限制距离
                        # 例如: ["China", "乐乐"]
                        
                        # 检查是否存在按顺序出现的匹配
                        has_ordered_match = True
                        current_pos = -1
                        
                        for kw in combo_keywords_lower:
                            # 找到当前关键词在当前位置之后的第一个匹配
                            found = False
                            for start_pos, end_pos in keyword_positions[kw]:
                                if start_pos > current_pos:
                                    current_pos = end_pos
                                    found = True
                                    break
                            
                            if not found:
                                has_ordered_match = False
                                break
                        
                        if has_ordered_match:
                            base_score += 1.0
            else:
                # 5. 普通关键词（字符串）: 匹配则得1分
                kw_str = str(keyword_entry).lower()
                if kw_str in keyword_positions:
                    base_score += 1.0
        
        # 应用优先级权重：(1 + priority * 0.1)
        priority_weight = (1 + priority * 0.1)
        final_score = base_score * priority_weight
        
        logging.debug(f"分类 {category} 评分详情: 基础分数={base_score:.2f}, 优先级={priority}, 最终分数={final_score:.2f}")
        
        return final_score
    
    def handle(self, context: ClassificationContext) -> bool:
        """计算分类分数 - 过滤分数小于等于0的分类"""
        try:
            category_matches = context.metadata.get('keyword_matches', {})
            
            new_scores = {}
            for category in context.categories: # 只对未被排除的分类计算
                # 检查匹配结果是否存在，否则跳过
                if category not in category_matches:
                    continue
                
                matches = category_matches[category]
                score = self._calculate_category_score(category, matches)
                
                # 仅保留分数大于0的分类 (修复了优先级-1的问题，只要分数>0就留下)
                if score > 0:
                    new_scores[category] = score
            
            context.scores = new_scores
            context.categories = list(context.scores.keys())
            
            return self._call_next(context)
            
        except Exception as e:
            logging.error(f"分数计算失败: {e}")
            return False


class DecisionHandler(Handler):
    """5. 决策处理器"""
    
    def __init__(self, enable_user_interaction: bool = True, is_experimental_mode: bool = False):
        super().__init__()
        self.enable_user_interaction = enable_user_interaction
        self.is_experimental_mode = is_experimental_mode
    
    def handle(self, context: ClassificationContext) -> bool:
        """处理最终决策"""
        try:
            if not context.categories:
                # 无匹配分类
                context.metadata['result'] = ClassificationResult(status='unclassified', folder_name=context.folder_name)
                return True
            
            # 按分数排序
            sorted_categories = sorted(context.scores.items(), key=lambda x: x[1], reverse=True)
            best_category, best_score = sorted_categories[0]
            scores_detail = {cat: score for cat, score in sorted_categories}
            
            # 获取所有分数大于0的分类
            all_positive_categories = [cat for cat, score in sorted_categories if score > 0]
            
            # 决策逻辑：根据用户交互设置决定是否需要人工判断
            if self.enable_user_interaction and len(all_positive_categories) > 1:
                # 启用用户交互且有多个正分分类：等待人工判断，列出所有分数大于0的分类
                status = 'pending'
                if self.is_experimental_mode:
                     print(f"[实验模式] 文件夹: '{context.folder_name}' -> 待定 (选项: {all_positive_categories}) (分数: {[context.scores[cat] for cat in all_positive_categories]})")
                
                context.metadata['result'] = ClassificationResult(
                    status=status, categories=all_positive_categories, folder_name=context.folder_name,
                    score=best_score, scores_detail=scores_detail
                )
            elif len(sorted_categories) == 1 or sorted_categories[0][1] > sorted_categories[1][1]:
                # 唯一最高分或禁用用户交互：直接选择
                status = 'classified'
                if self.is_experimental_mode:
                    print(f"[实验模式] 文件夹: '{context.folder_name}' -> 分类: {best_category} (分数: {best_score:.2f})")
                
                context.metadata['result'] = ClassificationResult(
                    status=status, category=best_category, folder_name=context.folder_name,
                    score=best_score, scores_detail=scores_detail
                )
            else:
                # 多个相同最高分且禁用用户交互：自动选择第一个
                tied_categories = [cat for cat, score in sorted_categories if score == best_score]
                status = 'classified'
                if self.is_experimental_mode:
                    # 优化输出，突出显示预期分类
                    if '清水由乃' in tied_categories[0]:
                        print(f"[实验模式] 文件夹: '{context.folder_name}' -> 自动选择: {tied_categories[0]} (平局分数: {best_score:.2f}) - 已匹配到预期分类")
                    else:
                        print(f"[实验模式] 文件夹: '{context.folder_name}' -> 自动选择: {tied_categories[0]} (平局分数: {best_score:.2f})，选项: {tied_categories}")
                
                context.metadata['result'] = ClassificationResult(
                    status=status, category=tied_categories[0], folder_name=context.folder_name,
                    score=best_score, scores_detail=scores_detail
                )
            
            return True
            
        except Exception as e:
            logging.error(f"决策处理失败: {e}")
            return False


class ClassificationChain:
    """分类责任链 - 协调处理器"""
    
    def __init__(self):
        self.head: Optional[Handler] = None
    
    def add_handler(self, handler: Handler) -> 'ClassificationChain':
        """添加处理器"""
        if not self.head:
            self.head = handler
        else:
            current = self.head
            while current.next_handler:
                current = current.next_handler
            current.next_handler = handler
        return self
    
    def process(self, context: ClassificationContext) -> ClassificationResult:
        """处理分类请求"""
        if self.head and self.head.handle(context) and 'result' in context.metadata:
            return context.metadata['result']
        
        # 默认返回未分类
        return ClassificationResult(status='unclassified', folder_name=context.folder_name)


# ----------------------------------------------------------------------
# 5. Threading Workers (线程工作者)
# ----------------------------------------------------------------------

class ClassificationWorker(threading.Thread):
    """分类工作线程 - 仅负责分类计算"""
    
    def __init__(self, task_queue: Queue, result_queue: Queue, chain: ClassificationChain):
        super().__init__(daemon=True)
        self.task_queue = task_queue
        self.result_queue = result_queue
        self.chain = chain
    
    def run(self):
        """工作线程主循环"""
        while True:
            folder_name = 'unknown'
            try:
                task = self.task_queue.get()
                if task is None:  # 结束信号
                    break
                
                folder_name, folder_path = task
                context = ClassificationContext(folder_name, folder_path)
                result = self.chain.process(context)
                self.result_queue.put((result, context))
                
            except Exception as e:
                logging.error(f"分类工作线程错误 (文件夹: {folder_name}): {e}")
                self.result_queue.put((
                    ClassificationResult(status='error', folder_name=folder_name, error_message=str(e)),
                    None
                ))
            finally:
                self.task_queue.task_done()


class IOWorker(threading.Thread):
    """IO工作线程 - 负责文件移动等IO操作"""
    
    def __init__(self, move_queue: Queue):
        super().__init__(daemon=True)
        self.move_queue = move_queue
    
    def run(self):
        """IO工作线程主循环"""
        while True:
            try:
                task = self.move_queue.get()
                if task is None:  # 结束信号
                    break
                
                if isinstance(task, MoveTask):
                    self._process_move_task(task)
                
            except Exception as e:
                logging.error(f"IO工作线程错误: {e}")
            finally:
                self.move_queue.task_done()
    
    def _process_move_task(self, task: MoveTask):
        """处理移动任务"""
        try:
            task.target_path.parent.mkdir(parents=True, exist_ok=True)
            
            if task.source_path.exists():
                shutil.move(str(task.source_path), str(task.target_path))
                logging.info(f"已移动: {task.folder_name} -> {task.category} 到 {task.target_path}")
            else:
                logging.warning(f"源文件夹不存在: {task.source_path}")
                
        except Exception as e:
            logging.error(f"移动文件夹失败 {task.folder_name}: {e}")


# ----------------------------------------------------------------------
# 6. Main Classifier (主分类器)
# ----------------------------------------------------------------------

class FolderClassifierV5Refactored:
    """文件夹分类器 V5 - 责任链模式重构版"""
    
    def __init__(self, config_path: str = "classification_rules.json", 
                 enable_parallel: bool = True, max_workers: int = 4,
                 io_workers: int = 2, is_experimental_mode: bool = False,
                 enable_user_interaction: bool = True, output_root: Optional[Path] = None):
        """
        初始化分类器
        
        Args:
            config_path: 配置文件路径
            enable_parallel: 是否启用并行处理
            max_workers: 最大分类工作线程数
            io_workers: IO工作线程数
            is_experimental_mode: 是否启用实验模式（只输出结果，不执行文件移动）
            enable_user_interaction: 是否启用用户交互，当有多个正分分类时等待人工判断
            output_root: 输出根目录，优先于配置文件中的设置
        """
        self.enable_parallel = enable_parallel
        self.max_workers = max_workers
        self.io_workers = io_workers
        self.is_experimental_mode = is_experimental_mode
        self.enable_user_interaction = enable_user_interaction
        self.config_path = Path(config_path)

        # 1. 加载配置
        self.config = self._load_config()
        self.categories, self.exclude_keywords, config_output_root = self._parse_config(self.config)
        
        # 优先使用外部指定的输出根目录，否则使用配置文件中的设置
        self.output_root = output_root if output_root else config_output_root
        
        # 2. 构建拼音组合关键字
        self.categories = self._build_pinyin_combo_keywords(self.categories)
        
        # 3. 初始化 AC 自动机和构建
        self.ac_matcher = AhoCorasickMatcher()
        self.ac_matcher.category_config = self.categories
        self._build_ac_matcher()
        
        # 4. 初始化责任链
        self.chain = self._build_chain()
        
        # 5. 初始化线程池和队列
        self.task_queue = Queue()
        self.result_queue = Queue()
        self.move_queue = Queue()
        self.classification_workers: List[ClassificationWorker] = []
        self.io_workers_ = []
        self._init_workers()
        
    def _init_workers(self):
        """初始化工作线程"""
        # 初始化分类工作线程
        for _ in range(self.max_workers):
            worker = ClassificationWorker(self.task_queue, self.result_queue, self.chain)
            self.classification_workers.append(worker)
            worker.start()
        
        # 初始化IO工作线程
        for _ in range(self.io_workers):
            worker = IOWorker(self.move_queue)
            self.io_workers_.append(worker)
            worker.start()
            
    def _build_pinyin_combo_keywords(self, categories: Any) -> Any:
        """
        构建拼音组合关键字
        
        规则：
        1. 对于中文普通关键字（字符串类型），将其拆分为单字，为每个单字生成对应的拼音，然后创建由这些拼音组成的数组作为组合关键字
        2. 对于包含中文的组合关键字（数组类型），若数组中包含中文元素，将中文元素转换为对应的拼音数组，生成包含拼音元素的新组合关键字数组
        
        Args:
            categories: 原始分类配置（支持字典或列表格式）
            
        Returns:
            扩展后的分类配置，包含拼音组合关键字
        """
        # 检查pypinyin是否可用
        try:
            from pypinyin import lazy_pinyin, Style
            pypinyin_available = True
        except ImportError:
            logging.warning("未找到pypinyin库，将跳过拼音组合关键字构建")
            return categories
        
        # 初始化预处理处理器以使用其拼音转换功能
        preprocessor = PreprocessingHandler()
        if not preprocessor.pypinyin_available:
            return categories
        
        # 遍历所有分类，处理关键词
        updated_categories = categories.copy()
        
        if isinstance(updated_categories, dict):
            # 字典格式：{"category_name": {"keywords": [...]}}
            for category_name, category_config in updated_categories.items():
                if not isinstance(category_config, dict):
                    continue
                    
                keywords = category_config.get('keywords', [])
                if not keywords:
                    continue
                    
                # 构建拼音组合关键字
                extended_keywords = preprocessor._build_pinyin_combo_keywords(keywords)
                category_config['keywords'] = extended_keywords
        elif isinstance(updated_categories, list):
            # 列表格式：[{"name": "category_name", "keywords": [...]}]
            for category in updated_categories:
                if not isinstance(category, dict):
                    continue
                    
                keywords = category.get('keywords', [])
                if not keywords:
                    continue
                    
                # 构建拼音组合关键字
                extended_keywords = preprocessor._build_pinyin_combo_keywords(keywords)
                category['keywords'] = extended_keywords
        
        return updated_categories
        
    def _load_config(self) -> Dict[str, Any]:
        """加载配置文件"""
        try:
            with open(self.config_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        except FileNotFoundError:
            logging.error(f"配置文件不存在: {self.config_path}")
            raise
        except json.JSONDecodeError:
            logging.error(f"配置文件格式错误: {self.config_path}")
            raise
        
    def _parse_config(self, config: Dict[str, Any]) -> Tuple[Any, Dict[str, List[str]], Path]:
        """解析配置"""
        categories = config.get('categories', {})
        exclude_keywords = config.get('exclude_keywords', {})
        # 将默认输出根目录设置为D:\Download\classify
        output_root = Path(config.get('output_root', 'D:\\Download\\classify'))
        return categories, exclude_keywords, output_root
        
    def _build_ac_matcher(self):
        """构建AC自动机"""
        if isinstance(self.categories, dict):
            # 字典格式：{"category_name": {"keywords": [...]}}
            for category_name, category_config in self.categories.items():
                keywords = category_config.get('keywords', [])
                
                for keyword_entry in keywords:
                    if isinstance(keyword_entry, str):
                        # 普通关键字：直接添加到AC自动机
                        self.ac_matcher.add_keyword(keyword_entry, category_name)
                    elif isinstance(keyword_entry, list):
                        # 组合关键字：将其中的每个词也添加到AC自动机，以便后续分数计算阶段使用
                        for keyword in keyword_entry:
                            if isinstance(keyword, str):
                                self.ac_matcher.add_keyword(keyword, category_name)
        elif isinstance(self.categories, list):
            # 列表格式：[{"name": "category_name", "keywords": [...]}]
            for category_config in self.categories:
                category_name = category_config.get('name', 'unknown')
                keywords = category_config.get('keywords', [])
                
                for keyword_entry in keywords:
                    if isinstance(keyword_entry, str):
                        # 普通关键字：直接添加到AC自动机
                        self.ac_matcher.add_keyword(keyword_entry, category_name)
                    elif isinstance(keyword_entry, list):
                        # 组合关键字：将其中的每个词也添加到AC自动机，以便后续分数计算阶段使用
                        for keyword in keyword_entry:
                            if isinstance(keyword, str):
                                self.ac_matcher.add_keyword(keyword, category_name)
        
        # 构建失败指针
        self.ac_matcher.build_failure_links()
        
    def _build_chain(self) -> ClassificationChain:
        """构建责任链"""
        chain = ClassificationChain()
        
        # 创建处理器
        preprocessing_handler = PreprocessingHandler()
        keyword_matching_handler = KeywordMatchingHandler(self.ac_matcher)
        exclusion_filter_handler = ExclusionFilterHandler(self.exclude_keywords)
        score_calculation_handler = ScoreCalculationHandler(self.categories)
        decision_handler = DecisionHandler(enable_user_interaction=self.enable_user_interaction, is_experimental_mode=self.is_experimental_mode)
        
        # 构建责任链
        chain.add_handler(preprocessing_handler)
        preprocessing_handler.set_next(keyword_matching_handler)
        keyword_matching_handler.set_next(exclusion_filter_handler)
        exclusion_filter_handler.set_next(score_calculation_handler)
        score_calculation_handler.set_next(decision_handler)
        
        return chain
        
    def classify_folder(self, folder_path: Path) -> ClassificationResult:
        """分类单个文件夹"""
        context = ClassificationContext(folder_path.name, folder_path)
        return self.chain.process(context)
        
    def classify_folders(self, target_path: Path) -> List[ClassificationResult]:
        """分类目标路径下的所有文件夹"""
        results = []
        
        # 遍历目标文件夹，提交分类任务
        for item in target_path.iterdir():
            if item.is_dir():
                self.task_queue.put((item.name, item))
        
        # 等待所有分类任务完成
        self.task_queue.join()
        
        # 收集分类结果
        while not self.result_queue.empty():
            result, _ = self.result_queue.get()
            results.append(result)
            self.result_queue.task_done()
        
        return results
        
    def move_classified_folders(self, results: List[ClassificationResult], source_path: Path, force_move: bool = False):
        """移动已分类的文件夹
        
        Args:
            results: 分类结果列表
            source_path: 源文件夹路径
            force_move: 是否强制移动，即使在实验模式下
        """
        for result in results:
            if result.status == 'classified' and result.category:
                source_folder = source_path / result.folder_name
                target_folder = self.output_root / result.category / result.folder_name
                
                if self.is_experimental_mode and not force_move:
                    logging.info(f"[实验模式] 准备移动: {result.folder_name} -> {result.category} 到 {target_folder}")
                else:
                    # 提交移动任务
                    move_task = MoveTask(source_folder, target_folder, result.folder_name, result.category)
                    self.move_queue.put(move_task)
        
        # 等待所有移动任务完成
        self.move_queue.join()
        
    def stop(self):
        """停止所有工作线程"""
        # 发送结束信号
        for _ in range(self.max_workers):
            self.task_queue.put(None)
        
        for _ in range(self.io_workers):
            self.move_queue.put(None)
        
        # 等待所有线程结束
        for worker in self.classification_workers:
            worker.join()
        
        for worker in self.io_workers_:
            worker.join()
            
    def print_full_config(self):
        """打印完整配置信息，包括配置文件中的规则"""
        logging.info("\n=== 分类器完整配置信息 ===")
        logging.info(f"配置文件路径: {self.config_path}")
        logging.info(f"输出根目录: {self.output_root}")
        logging.info(f"并行处理: {'启用' if self.enable_parallel else '禁用'}")
        logging.info(f"分类工作线程数: {self.max_workers}")
        logging.info(f"IO工作线程数: {self.io_workers}")
        logging.info(f"实验模式: {'启用' if self.is_experimental_mode else '禁用'}")
        logging.info(f"用户交互: {'启用' if self.enable_user_interaction else '禁用'}")
        
        # 打印分类规则数量
        if isinstance(self.categories, dict):
            logging.info(f"分类规则数量: {len(self.categories)}")
        elif isinstance(self.categories, list):
            logging.info(f"分类规则数量: {len(self.categories)}")
        
        # 打印排除规则数量
        if isinstance(self.exclude_keywords, dict):
            logging.info(f"排除规则数量: {len(self.exclude_keywords)}")
        
        # 打印AC自动机关键词数量
        logging.info(f"AC自动机关键词数量: {len(self.ac_matcher.keywords)}")


# 主函数入口
if __name__ == "__main__":
    # 创建参数解析器
    parser = argparse.ArgumentParser(description="文件夹智能分类器 V5")
    
    # 核心功能参数
    parser.add_argument("--test", "-t", action="store_true", help="运行测试模式，验证拼音组合关键字生成功能")
    parser.add_argument("--config", "-c", default="classification_rules.json", help="指定配置文件路径")
    parser.add_argument("--target", "-d", help="指定目标文件夹路径（实际执行时必需）")
    parser.add_argument("--output", "-o", help="指定输出根目录")
    parser.add_argument("--experimental", "-e", action="store_true", help="启用实验模式，只输出结果不执行文件移动")
    parser.add_argument("--parallel", "-p", type=int, default=4, help="设置分类工作线程数")
    parser.add_argument("--io-workers", type=int, default=2, help="设置IO工作线程数")
    parser.add_argument("--no-interaction", action="store_true", help="禁用用户交互，当有多个正分分类时自动选择最高分")
    
    # 解析命令行参数
    args = parser.parse_args()
    
    # 测试模式
    if args.test:
        print("\n=== 拼音组合关键字生成测试 ===")
        
        # 示例配置
        test_config = {
            "name": "China@奶兔biubiu",
            "keywords": ["奶兔", ["奶兔", "biubiu"], "biubiu"]
        }
        
        # 1. 测试PreprocessingHandler的拼音组合关键字构建功能
        print("\n1. 测试PreprocessingHandler的拼音组合关键字构建功能")
        preprocessor = PreprocessingHandler()
        if preprocessor.pypinyin_available:
            # 测试普通关键词处理
            print("\n   --- 普通关键词处理测试 ---")
            keyword = "奶兔"
            pinyin_list = preprocessor._convert_chinese_to_pinyin(keyword)
            print(f"   输入关键词: {keyword}")
            print(f"   生成拼音数组: {pinyin_list}")
            
            # 测试组合关键词处理
            print("\n   --- 组合关键词处理测试 ---")
            combo_keyword = ["奶兔", "biubiu"]
            extended_keywords = preprocessor._build_pinyin_combo_keywords(combo_keyword)
            print(f"   输入组合关键词: {combo_keyword}")
            print(f"   扩展后关键词: {extended_keywords}")
            
            # 测试完整关键词列表处理
            print("\n   --- 完整关键词列表处理测试 ---")
            source_keywords = test_config["keywords"]
            extended_keywords = preprocessor._build_pinyin_combo_keywords(source_keywords)
            print(f"   输入关键词列表: {source_keywords}")
            print(f"   扩展后关键词列表: {extended_keywords}")
            
            # 验证生成的拼音组合关键字是否符合预期
            print("\n   --- 验证结果 ---")
            expected_keywords = ["奶兔", ["nai","tu"], ["奶兔", "biubiu"], ["nai","tu", "biubiu"], "biubiu"]
            print(f"   预期输出: {expected_keywords}")
            
            # 检查是否包含预期的拼音组合关键字
            contains_nai_tu = ["nai","tu"] in extended_keywords
            contains_nai_tu_biubiu = ["nai","tu", "biubiu"] in extended_keywords
            print(f"   包含 [nai,tu]: {contains_nai_tu}")
            print(f"   包含 [nai,tu,biubiu]: {contains_nai_tu_biubiu}")
            
            if contains_nai_tu and contains_nai_tu_biubiu:
                print("   ✓ 测试通过: 拼音组合关键字生成符合预期")
            else:
                print("   ✗ 测试失败: 拼音组合关键字生成不符合预期")
                print("   实际生成: {}".format(extended_keywords))
                print("   预期包含: {}".format(expected_keywords))
        else:
            print("   pypinyin库不可用，跳过测试")
        
        # 2. 测试拼音转换功能
        print("\n2. 测试拼音转换功能")
        if preprocessor.pypinyin_available:
            # 测试多个中文词语的拼音转换
            test_words = ["测试", "中文", "拼音", "转换"]
            print("\n   --- 中文词语拼音转换测试 ---")
            for word in test_words:
                pinyin_list = preprocessor._convert_chinese_to_pinyin(word)
                print(f"   {word} -> {pinyin_list}")
        else:
            print("   pypinyin库不可用，跳过测试")
        
        print("\n=== 测试完成 ===")
    
    # 实际执行模式
    else:
        # 验证必需参数
        if not args.target:
            parser.error("实际执行模式下必需指定目标文件夹路径 (-d/--target)")
        
        # 构建目标路径和配置
        target_path = Path(args.target)
        
        # 验证目标路径是否存在
        if not target_path.exists() or not target_path.is_dir():
            parser.error(f"目标文件夹不存在或不是目录: {target_path}")
        
        logging.info(f"开始文件夹分类任务")
        logging.info(f"目标文件夹: {target_path}")
        logging.info(f"配置文件: {args.config}")
        logging.info(f"实验模式: {'启用' if args.experimental else '禁用'}")
        
        try:
            # 处理输出根目录
            output_root = None
            if args.output:
                output_root = Path(args.output)
            
            # 创建分类器实例
            classifier = FolderClassifierV5Refactored(
                config_path=args.config,
                enable_parallel=True,
                max_workers=args.parallel,
                io_workers=args.io_workers,
                is_experimental_mode=args.experimental,
                enable_user_interaction=not args.no_interaction,
                output_root=output_root
            )
            
            # 打印配置信息
            print("\n=== 分类器配置信息 ===")
            print(f"配置文件路径: {args.config}")
            print(f"目标文件夹: {target_path}")
            print(f"输出根目录: {classifier.output_root}")
            print(f"实验模式: {'启用' if args.experimental else '禁用'}")
            print(f"并行处理: {'启用' if classifier.enable_parallel else '禁用'}")
            print(f"分类工作线程数: {classifier.max_workers}")
            print(f"IO工作线程数: {classifier.io_workers}")
            print(f"用户交互: {'启用' if classifier.enable_user_interaction else '禁用'}")
            print(f"命令行指定输出: {'是' if args.output else '否'}")
            
            # 打印完整配置信息到日志
            classifier.print_full_config()
            
            # 分类目标文件夹
            logging.info("开始分类计算...")
            results = classifier.classify_folders(target_path)
            
            # 1. 统计初始分类结果
            classified_count = sum(1 for r in results if r.status == 'classified')
            unclassified_count = sum(1 for r in results if r.status == 'unclassified')
            pending_count = sum(1 for r in results if r.status == 'pending')
            error_count = sum(1 for r in results if r.status == 'error')
            
            logging.info(f"分类计算完成 - 总计: {len(results)}, 已自动分类: {classified_count}, 未分类: {unclassified_count}, 需人工判断: {pending_count}, 错误: {error_count}")
            
            # 2. 输出自动分类结果
            if classified_count > 0:
                print("\n=== 自动分类结果 ===")
                for result in results:
                    if result.status == 'classified':
                        print(f"文件夹: '{result.folder_name}' -> 分类: {result.category} (分数: {result.score:.2f})")
            
            # 3. 统一处理需要人工判断的文件夹
            pending_results = [r for r in results if r.status == 'pending' and r.categories]
            if pending_results:
                print(f"\n=== 人工判断处理 ===")
                print(f"发现 {len(pending_results)} 个文件夹需要人工判断")
                print(f"是否要手动处理这些文件夹？")
                print(f"可用指令：是/执行/确认/y (手动处理)，否/取消/n (跳过，保持待定状态)")
                
                # 输入处理循环
                while True:
                    user_input = input("请输入指令: ").strip().lower()
                    
                    # 解析用户输入
                    if user_input in ['是', '执行', '确认', 'y', 'yes']:
                        # 手动处理待定分类
                        print(f"\n=== 开始人工判断 ===")
                        
                        # 遍历所有需要人工判断的结果
                        processed_count = 0
                        total_pending = len(pending_results)
                        
                        for i, result in enumerate(pending_results):
                            if result.status != 'pending' or not result.categories:
                                continue
                            
                            print(f"\n[{i+1}/{total_pending}] 文件夹: '{result.folder_name}'")
                            print(f"可用分类选项:")
                            for j, category in enumerate(result.categories):
                                print(f"  {j+1}. {category} (分数: {result.scores_detail[category]:.2f})")
                            print(f"  0. 跳过，保持待定状态")
                            print(f"  -1. 取消所有人工判断")
                            
                            # 等待用户选择
                            while True:
                                choice_input = input("请选择分类 (输入数字，s 待定，q 取消所有): ").strip().lower()
                                
                                # 处理特殊指令
                                if choice_input == 's':
                                    # 输入 's'，保持待定状态
                                    print(f"跳过文件夹 '{result.folder_name}'，保持待定状态")
                                    break
                                elif choice_input == 'q':
                                    # 输入 'q'，取消所有人工判断
                                    print(f"\n=== 取消所有人工判断 ===")
                                    # 使用特殊值-999标记取消所有
                                    choice = -999
                                    break
                                
                                # 验证输入是否为数字
                                if not choice_input.isdigit() and choice_input != '-1':
                                    print(f"无效输入: '{choice_input}'。请输入数字，或 s (待定)，或 q (取消所有)。")
                                    continue
                                
                                choice = int(choice_input)
                                
                                # 处理选择
                                if choice == 0:
                                    # 跳过，保持待定状态
                                    print(f"跳过文件夹 '{result.folder_name}'，保持待定状态")
                                    break
                                elif choice == -1:
                                    # 取消所有人工判断
                                    print(f"\n=== 取消所有人工判断 ===")
                                    break
                                elif 1 <= choice <= len(result.categories):
                                    # 选择了一个分类
                                    selected_category = result.categories[choice-1]
                                    print(f"选择分类: {selected_category}")
                                    
                                    # 更新结果状态
                                    result.status = 'classified'
                                    result.category = selected_category
                                    processed_count += 1
                                    break
                                else:
                                    # 无效选择
                                    print(f"无效选择: {choice}。请输入 0 到 {len(result.categories)} 之间的数字，或 s (待定)，或 q (取消所有)。")
                            
                            # 如果用户取消了所有人工判断，跳出外层循环
                            if choice in [-1, -999]:
                                break
                        
                        # 重新统计分类结果
                        classified_count = sum(1 for r in results if r.status == 'classified')
                        pending_count = sum(1 for r in results if r.status == 'pending')
                        
                        print(f"\n=== 人工判断完成 ===")
                        print(f"已处理 {processed_count} 个文件夹")
                        print(f"当前分类结果: 已分类: {classified_count}, 需人工判断: {pending_count}")
                        break
                    elif user_input in ['否', '取消', 'n', 'no']:
                        # 跳过手动处理，保持待定状态
                        print(f"\n=== 跳过人工判断 ===")
                        print(f"{len(pending_results)} 个文件夹将保持待定状态")
                        break
                    else:
                        # 无效指令
                        print(f"无效指令: '{user_input}'。可用指令：是/执行/确认/y (手动处理)，否/取消/n (跳过)")
            
            # 移动已分类文件夹
            if classified_count > 0:
                if args.experimental:
                    # 实验模式：询问用户是否执行实际操作
                    print(f"\n=== 实验模式操作确认 ===")
                    print(f"已完成分类，共 {classified_count} 个文件夹将被移动")
                    print(f"是否执行实际文件移动操作？")
                    print(f"可用指令：是/执行/确认/y (执行操作)，否/取消/n (取消操作)，详细/info (查看详细信息)")
                    
                    # 输入处理循环
                    while True:
                        user_input = input("请输入指令: ").strip().lower()
                        
                        # 解析用户输入
                        if user_input in ['是', '执行', '确认', 'y', 'yes']:
                            # 明确执行指令
                            print(f"\n=== 执行文件移动操作 ===")
                            logging.info("实验模式下用户确认执行文件移动")
                            # 传递force_move=True，强制在实验模式下执行移动操作
                            classifier.move_classified_folders(results, target_path, force_move=True)
                            logging.info("文件夹移动完成")
                            print(f"操作完成：已移动 {classified_count} 个文件夹")
                            break
                        elif user_input in ['否', '取消', 'n', 'no']:
                            # 明确取消指令
                            print(f"\n=== 取消文件移动操作 ===")
                            logging.info("实验模式下用户取消执行文件移动")
                            print(f"操作已取消，未执行任何文件移动")
                            break
                        elif user_input in ['详细', 'info', 'details']:
                            # 查看详细信息
                            print(f"\n=== 操作详情 ===")
                            print(f"目标文件夹: {target_path}")
                            print(f"配置文件: {args.config}")
                            print(f"分类结果: 总计 {len(results)}, 已分类 {classified_count}, 未分类 {unclassified_count}")
                            print(f"将移动 {classified_count} 个文件夹到指定分类目录")
                            print(f"可用指令：是/执行/确认/y (执行操作)，否/取消/n (取消操作)")
                        else:
                            # 模糊指令，需要二次确认
                            print(f"\n=== 指令不明确，需要二次确认 ===")
                            print(f"您输入的指令 '{user_input}' 不够明确")
                            print(f"请明确输入：是/执行/确认/y (执行操作)，否/取消/n (取消操作)")
                else:
                    # 正常模式：直接执行
                    logging.info("开始移动已分类文件夹...")
                    classifier.move_classified_folders(results, target_path)
                    logging.info("文件夹移动完成")
            
            # 停止分类器
            classifier.stop()
            logging.info("分类任务完成")
            
        except Exception as e:
            logging.error(f"分类任务失败: {e}")
            import traceback
            traceback.print_exc()