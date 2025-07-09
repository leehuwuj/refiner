'use client';

import { emit, listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import { RiTranslate } from 'react-icons/ri';

export default function SelectionIcon() {
    const [selectedText, setSelectedText] = useState('');

    useEffect(() => {
        // Listen for selected text updates
        const unlisten = listen('set-selected-text', (event) => {
            setSelectedText(event.payload as string);
        });

        return () => {
            unlisten.then(fn => fn()); // Cleanup listener
        };
    }, []);

    const handleClick = async (e: React.MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();

        try {
            // Emit the event with the selected text
            await emit('icon-clicked', selectedText);
        } catch (error) {
            console.error('Error emitting icon-clicked event:', error);
        }
    };

    const handleMouseDown = (e: React.MouseEvent) => {
        // Prevent the mousedown from bubbling to avoid interfering with other interactions
        e.stopPropagation();
    };

    return (
        <div
            className="w-6 h-6 rounded-full bg-blue-200 hover:bg-blue-300 cursor-pointer flex items-center justify-center shadow-lg transition-colors duration-200"
            onClick={handleClick}
            onMouseDown={handleMouseDown}
            role="button"
            tabIndex={0}
            aria-label="Translate selected text"
        >
            <RiTranslate className="text-blue-600" size={16} />
        </div>
    );
} 