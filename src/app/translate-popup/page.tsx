import dynamic from 'next/dynamic'
import { Providers } from '../provider';

const TranslatePopupClient = dynamic(
    () => import('./translate-popup-client'),
);

export default function TranslatePopup() {
    return (
        <Providers>
            <TranslatePopupClient />
        </Providers>
    );
}
